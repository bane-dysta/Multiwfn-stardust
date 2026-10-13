#pragma once
#include <cstdlib>
#include <cstdint>
#include <string>
#include <stdexcept>
#include <cerrno>
#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#include <fcntl.h>
#endif
namespace SessionExecution {
inline std::string currentFile;
inline bool enabled() { return std::getenv("BANEWFN_SESSION_MODE") != nullptr; }
inline void transfer(bool writing,void *data,size_t size) {
#ifdef _WIN32
    static HANDLE channels[2]={nullptr,nullptr};
    if(!channels[0]) {
        uint64_t raw[2]; auto*p=reinterpret_cast<unsigned char*>(raw);size_t left=sizeof(raw);
        while(left) { DWORD n=0;if(!ReadFile(GetStdHandle(STD_INPUT_HANDLE),p,static_cast<DWORD>(left),&n,nullptr)||!n) throw std::runtime_error("Session bootstrap disconnected");p+=n;left-=n; }
        channels[0]=reinterpret_cast<HANDLE>(static_cast<uintptr_t>(raw[0]));
        channels[1]=reinterpret_cast<HANDLE>(static_cast<uintptr_t>(raw[1]));
    }
    auto*p=static_cast<unsigned char*>(data);
    while(size) {
        DWORD n=0;
        BOOL ok=writing?WriteFile(channels[1],p,static_cast<DWORD>(size),&n,nullptr):ReadFile(channels[0],p,static_cast<DWORD>(size),&n,nullptr);
        if(!ok || !n) throw std::runtime_error("Session host disconnected");
        p+=n;size-=n;
    }
#else
    const char *value=std::getenv(writing?"BANEWFN_SESSION_WRITE":"BANEWFN_SESSION_READ");
    if (!value) throw std::runtime_error("Session channel is unavailable");
    const int fd=std::stoi(value);
    if (fcntl(fd,F_SETFD,FD_CLOEXEC)<0) throw std::runtime_error("Invalid session channel");
    auto *p=static_cast<unsigned char*>(data);
    while (size) {
        ssize_t n=writing ? ::write(fd,p,size) : ::read(fd,p,size);
        if(n<0 && errno==EINTR) continue;
        if(n<=0) throw std::runtime_error("Session host disconnected");
        p+=n;size-=static_cast<size_t>(n);
    }
#endif
}
inline void number(uint32_t value) {
    unsigned char bytes[4];
    for(unsigned i=0;i<4;i++) bytes[i]=static_cast<unsigned char>(value>>(8*i));
    transfer(true,bytes,4);
}
inline void text(const std::string &value) {
    number(static_cast<uint32_t>(value.size()));
    transfer(true,const_cast<char*>(value.data()),value.size());
}
inline bool checkpoint() {
    char kind='P';transfer(true,&kind,1);transfer(false,&kind,1);return kind==0;
}
inline bool execute(const std::string &commands,const std::string &log,const std::string &input,int cores,bool reload=false) {
    char kind=reload?'L':'B';transfer(true,&kind,1);number(static_cast<uint32_t>(cores));
    text(commands);text(log);text(input);
    transfer(false,&kind,1);
    unsigned char bytes[4];transfer(false,bytes,4);
    uint32_t size=0;for(unsigned i=0;i<4;i++) size|=static_cast<uint32_t>(bytes[i])<<(8*i);
    if(size>32768) throw std::runtime_error("Invalid current file response");
    currentFile.resize(size);transfer(false,currentFile.data(),size);
    return kind==0;
}
inline std::string withoutFinalQuit(std::string commands) {
    // Remove only generated termination; literal raw text is not rewritten.
    if(commands.size()>=2 && commands.compare(commands.size()-2,2,"q\n")==0 &&
       (commands.size()==2 || commands[commands.size()-3]=='\n')) commands.resize(commands.size()-2);
    return commands;
}
}
