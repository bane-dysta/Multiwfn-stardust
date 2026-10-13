/* Linux managed-session input scheduling. Scientific menus remain in Fortran.
 * Private inherited socket protocol, version 1, little endian:
 * event: kind ('R' ready, 'D' done, 'E' rejected), consumed:u64, filename:string
 * command: 'G' enter GUI, 'Q' close, 'B' end:u64 cores:u32 cwd:string log:string
 * Strings are u32 byte count + UTF-8. No listening socket or runtime files. */
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#if defined(__linux__)
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <sys/stat.h>
static int channel = -1, saved_output = -1;
static uint64_t block_end;
static int block_active, saved_silent;
static char *saved_directory;
static void transfer(int writing, void *data, size_t size) {
    unsigned char *p = data;
    while (size) {
        ssize_t n = writing ? write(channel, p, size) : read(channel, p, size);
        if (n < 0 && errno == EINTR) continue;
        if (n <= 0) _exit(74);
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
static char *string_read(void) {
    uint64_t n=number(0,0,4);
    if (n>32768) _exit(74);
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
    const char *value=getenv("MULTIWFN_SESSION_FD");
    if (!value) return -1;
    char *end=NULL;
    long fd=strtol(value,&end,10);
    if (!*value || *end || fd<3 || fd>INT_MAX || fcntl((int)fd,F_GETFD)<0) _exit(74);
    channel=(int)fd;
    if (fcntl(channel,F_SETFD,FD_CLOEXEC)<0) _exit(74);
    return dup(STDIN_FILENO);
}
int mwfn_session_managed(void) { return channel>=0; }
int mwfn_session_active(void) { return block_active; }
int64_t mwfn_session_end(void) { return (int64_t)block_end; }
int mwfn_session_absolute(char *file,int capacity) {
    char *resolved=realpath(file,NULL);
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
        if (chdir(saved_directory)!=0) _exit(74);
        free(saved_directory); saved_directory=NULL;
    }
    *silent=saved_silent; block_active=0;
}
int mwfn_session_menu(int64_t position,const char *file,int length,int *silent,int *cores,int rejected) {
    if (channel<0) return 0;
    if (block_active && (uint64_t)position<block_end && !rejected) return 0;
    char kind=block_active ? (rejected ? 'E':'D'):'R';
    if (block_active) {
        if ((uint64_t)position!=block_end) _exit(74);
        finish(silent);
    }
    event(kind,(uint64_t)position,file,length);
    for (;;) {
        char command; transfer(0,&command,1);
        if (command=='G') return 1;
        if (command=='Q') return 2;
        if (command!='B') _exit(74);
        block_end=number(0,0,8);
        uint64_t threads=number(0,0,4);
        char *directory=string_read(),*log=string_read();
        if (block_end<=(uint64_t)position || threads<1 || threads>1024) _exit(74);
        saved_directory=getcwd(NULL,0);
        int out=open(log,O_WRONLY|O_CREAT|O_APPEND|O_NOFOLLOW,0600);
        if (!saved_directory || out<0 || chdir(directory)!=0) _exit(74);
        saved_output=dup(STDOUT_FILENO);
        if (saved_output<0 || dup2(out,STDOUT_FILENO)<0) _exit(74);
        close(out); free(directory); free(log);
        saved_silent=*silent; *silent=1; *cores=(int)threads; block_active=1;
        return 0;
    }
}
#else
int mwfn_session_initialize(void) { return -1; }
int mwfn_session_managed(void) { return 0; }
int mwfn_session_active(void) { return 0; }
int64_t mwfn_session_end(void) { return 0; }
int mwfn_session_absolute(char *file,int capacity) { (void)capacity; return (int)strlen(file); }
int mwfn_session_menu(int64_t p,const char*f,int l,int*s,int*c,int r) {
    (void)p;(void)f;(void)l;(void)s;(void)c;(void)r;return 0;
}
#endif
