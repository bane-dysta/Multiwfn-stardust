#pragma once
#include <cstdlib>
#include <cstdint>
#include <string>
#include <stdexcept>
#include <cerrno>
#if defined(__linux__)
#include <unistd.h>
#include <fcntl.h>
#endif
namespace SessionExecution {
inline std::string currentFile;
inline bool enabled() { return std::getenv("BANEWFN_SESSION_MODE") != nullptr; }
inline void transfer(bool writing,void *data,size_t size) {
#if defined(__linux__)
    const char *value=std::getenv("BANEWFN_SESSION_FD");
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
#else
    (void)writing;(void)data;(void)size;
    throw std::runtime_error("Current-session execution requires Linux");
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
