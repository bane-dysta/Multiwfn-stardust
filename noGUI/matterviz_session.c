/* Managed-session input scheduling. Scientific menus remain in Fortran.
 * Private pipe protocol, version MWS2, little endian:
 * event: kind ('R' ready, 'D' done, 'E' rejected), block id:u64, filename:string
 * command: 'G' enter GUI, 'Q' close, 'B' id:u64 cores:u32 cwd/log/commands:string
 * Strings are u32 byte count + UTF-8. Fortran owns its scratch input stream. */
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <fcntl.h>
#ifdef _WIN32
#include <windows.h>
#include <io.h>
#include <direct.h>
#define dup _dup
#define dup2 _dup2
#define close _close
#define STDOUT_FILENO 1
static HANDLE channel_read,channel_write;
static wchar_t *wide(const char *s) {
    int n=MultiByteToWideChar(CP_UTF8,MB_ERR_INVALID_CHARS,s,-1,NULL,0);
    wchar_t *p=n?calloc((size_t)n,sizeof(wchar_t)):NULL;
    if(!p || !MultiByteToWideChar(CP_UTF8,MB_ERR_INVALID_CHARS,s,-1,p,n)) _exit(74);
    return p;
}
static char *utf8(const wchar_t *s) {
    int n=WideCharToMultiByte(CP_UTF8,0,s,-1,NULL,0,NULL,NULL);
    char *p=n?calloc((size_t)n,1):NULL;
    if(!p || !WideCharToMultiByte(CP_UTF8,0,s,-1,p,n,NULL,NULL)) _exit(74);
    return p;
}
static char *current_directory(void) { wchar_t*w=_wgetcwd(NULL,0);if(!w)return NULL;char*p=utf8(w);free(w);return p; }
static int change_directory(const char*p) { wchar_t*w=wide(p);int r=_wchdir(w);free(w);return r; }
static int open_log(const char*p) { wchar_t*w=wide(p);int fd=_wopen(w,_O_WRONLY|_O_CREAT|_O_APPEND|_O_BINARY,0600);free(w);return fd; }
#else
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <sys/stat.h>
#endif
#ifndef _WIN32
static int channel_read=-1,channel_write=-1;
static char *current_directory(void) { return getcwd(NULL,0); }
static int change_directory(const char*p) { return chdir(p); }
static int open_log(const char*p) { return open(p,O_WRONLY|O_CREAT|O_APPEND|O_NOFOLLOW,0600); }
#endif
static int managed, saved_output = -1;
static uint64_t block_end, block_id;
static char *commands;
static int block_active, saved_silent;
static char *saved_directory;
static void transfer(int writing, void *data, size_t size) {
    unsigned char *p = data;
    while (size) {
#ifdef _WIN32
        DWORD n=0;
        BOOL ok=writing?WriteFile(channel_write,p,(DWORD)size,&n,NULL):ReadFile(channel_read,p,(DWORD)size,&n,NULL);
        if(!ok || !n) _exit(74);
#else
        ssize_t n = writing ? write(channel_write, p, size) : read(channel_read, p, size);
        if (n < 0 && errno == EINTR) continue;
        if (n <= 0) _exit(74);
#endif
        p += n; size -= (size_t)n;
    }
}
static uint64_t number(int writing, uint64_t value, size_t count) {
    unsigned char bytes[8];
    if (writing) for (size_t i=0;i<count;i++) bytes[i]=(unsigned char)(value>>(8*i));
    transfer(writing, bytes, count);
    if (!writing) { value=0; for(size_t i=0;i<count;i++) value|=(uint64_t)bytes[i]<<(8*i); }
    return value;
}
static char *string_read(size_t limit) {
    uint64_t n=number(0,0,4);
    if (n>limit) _exit(74);
    char *p=calloc((size_t)n+1,1);
    if (!p) _exit(74);
    transfer(0,p,(size_t)n);
    if (memchr(p,0,(size_t)n)) _exit(74);
    return p;
}
static void event(char kind,uint64_t position,const char *file,int length) {
    transfer(1,&kind,1); number(1,position,8);
    number(1,(uint64_t)length,4); transfer(1,(void*)file,(size_t)length);
}
int mwfn_session_initialize(void) {
#ifdef _WIN32
    if(!getenv("MULTIWFN_SESSION_BOOTSTRAP")) return -1;
    uint64_t handles[6];unsigned char*p=(unsigned char*)handles;size_t left=sizeof(handles);
    while(left) { DWORD n=0;if(!ReadFile(GetStdHandle(STD_INPUT_HANDLE),p,(DWORD)left,&n,NULL)||!n) _exit(74);p+=n;left-=n; }
    channel_read=(HANDLE)(uintptr_t)handles[0];channel_write=(HANDLE)(uintptr_t)handles[1];
    char gui[160];snprintf(gui,sizeof(gui),"%llu,%llu,%llu,%llu",(unsigned long long)handles[2],(unsigned long long)handles[3],(unsigned long long)handles[4],(unsigned long long)handles[5]);
    _putenv_s("MULTIWFN_SESSION_GUI_FDS",gui);
#else
    const char *value=getenv("MULTIWFN_SESSION_READ"),*writer=getenv("MULTIWFN_SESSION_WRITE");
    if (!value && !writer) return -1;
    if (!value || !writer) _exit(74);
    char *end=NULL;
    long fd=strtol(value,&end,10);
    if (!*value || *end || fd<3 || fd>INT_MAX || fcntl((int)fd,F_GETFD)<0) _exit(74);
    channel_read=(int)fd;
    fd=strtol(writer,&end,10);
    if(!*writer || *end || fd<3 || fd>INT_MAX) _exit(74);
    channel_write=(int)fd;
    if (fcntl(channel_read,F_SETFD,FD_CLOEXEC)<0 || fcntl(channel_write,F_SETFD,FD_CLOEXEC)<0) _exit(74);
#endif
    managed=1;transfer(1,"MWS2",4);return 0;
}
int mwfn_session_managed(void) { return managed; }
int mwfn_session_active(void) { return block_active; }
int64_t mwfn_session_end(void) { return (int64_t)block_end; }
void mwfn_session_set_end(int64_t position) { block_end=(uint64_t)position;free(commands);commands=NULL; }
void *mwfn_session_commands(void) { return commands; }
int mwfn_session_command_length(void) { return (int)strlen(commands); }
int mwfn_session_absolute(char *file,int capacity) {
#ifdef _WIN32
    wchar_t*w=wide(file),*full=_wfullpath(NULL,w,0);free(w);
    char*resolved=full?utf8(full):NULL;free(full);
#else
    char *resolved=realpath(file,NULL);
#endif
    if (!resolved) return (int)strlen(file);
    size_t length=strlen(resolved);
    if (length>=(size_t)capacity) { free(resolved); _exit(74); }
    memcpy(file,resolved,length+1); free(resolved); return (int)length;
}
static void finish(int *silent) {
    fflush(stdout);
    if (saved_output>=0) {
        if (dup2(saved_output,STDOUT_FILENO)<0) _exit(74);
        close(saved_output); saved_output=-1;
    }
    if (saved_directory) {
        if (change_directory(saved_directory)!=0) _exit(74);
        free(saved_directory); saved_directory=NULL;
    }
    *silent=saved_silent; block_active=0;
}
int mwfn_session_menu(int64_t position,const char *file,int length,int *silent,int *cores,int rejected) {
    if (!managed) return 0;
    if (block_active && (uint64_t)position<block_end && !rejected) return 0;
    char kind=block_active ? (rejected ? 'E':'D'):'R';
    if (block_active) {
        if ((uint64_t)position!=block_end) _exit(74);
        finish(silent);
    }
    event(kind,block_id,file,length);
    for (;;) {
        char command; transfer(0,&command,1);
        if (command=='G') return 1;
        if (command=='Q') return 2;
        if (command!='B') _exit(74);
        uint64_t id=number(0,0,8);
        uint64_t threads=number(0,0,4);
        char *directory=string_read(32768),*log=string_read(32768);
        commands=string_read(512*1024);
        size_t n=strlen(commands);
        if (!n || commands[n-1]!='\n' || id!=block_id+1 || threads<1 || threads>1024) _exit(74);
        saved_directory=current_directory();
        int out=open_log(log);
        if (!saved_directory || out<0 || change_directory(directory)!=0) _exit(74);
        saved_output=dup(STDOUT_FILENO);
        if (saved_output<0 || dup2(out,STDOUT_FILENO)<0) _exit(74);
        close(out); free(directory); free(log);
        saved_silent=*silent; *silent=1; *cores=(int)threads; block_active=1;
        block_id=id;return 3;
    }
}
