//! Private anonymous pipes. Windows endpoints are duplicated only into the
//! intended child, then delivered over its bootstrap stdin (never inherited).
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

pub struct Channel {
    reader: File,
    writer: File,
    timeout: Option<Duration>,
}
pub struct Endpoints(pub Vec<File>);

#[cfg(unix)]
pub fn raw(file: &File) -> u64 {
    use std::os::fd::AsRawFd;
    file.as_raw_fd() as u64
}
#[cfg(windows)]
pub fn raw(file: &File) -> u64 {
    use std::os::windows::io::AsRawHandle;
    file.as_raw_handle() as usize as u64
}
pub fn release(file: File) -> u64 {
    #[cfg(unix)]
    {
        use std::os::fd::IntoRawFd;
        file.into_raw_fd() as u64
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::IntoRawHandle;
        file.into_raw_handle() as usize as u64
    }
}
pub fn pipe() -> io::Result<(File, File)> {
    #[cfg(unix)]
    {
        use std::os::fd::FromRawFd;
        let mut fds = [-1; 2];
        if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let pair = unsafe { (File::from_raw_fd(fds[0]), File::from_raw_fd(fds[1])) };
        for fd in fds {
            if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(pair)
    }
    #[cfg(windows)]
    {
        use std::os::windows::io::FromRawHandle;
        use windows_sys::Win32::System::Pipes::CreatePipe;
        let (mut read, mut write) = (std::ptr::null_mut(), std::ptr::null_mut());
        if unsafe { CreatePipe(&mut read, &mut write, std::ptr::null(), 0) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { (File::from_raw_handle(read), File::from_raw_handle(write)) })
    }
}
impl Channel {
    pub fn pair() -> io::Result<(Self, Endpoints)> {
        let (reader, child_write) = pipe()?;
        let (child_read, writer) = pipe()?;
        Ok((
            Self {
                reader,
                writer,
                timeout: None,
            },
            Endpoints(vec![child_read, child_write]),
        ))
    }
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }
}
impl Read for Channel {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if let Some(timeout) = self.timeout {
            let start = Instant::now();
            loop {
                #[cfg(unix)]
                let ready = {
                    let mut poll = libc::pollfd {
                        fd: raw(&self.reader) as i32,
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    let result = unsafe { libc::poll(&mut poll, 1, 0) };
                    if result < 0 {
                        return Err(io::Error::last_os_error());
                    }
                    result > 0
                };
                #[cfg(windows)]
                let ready = {
                    let mut available = 0;
                    let result = unsafe {
                        windows_sys::Win32::System::Pipes::PeekNamedPipe(
                            raw(&self.reader) as usize as _,
                            std::ptr::null_mut(),
                            0,
                            std::ptr::null_mut(),
                            &mut available,
                            std::ptr::null_mut(),
                        )
                    };
                    if result == 0 {
                        return Err(io::Error::last_os_error());
                    }
                    available > 0
                };
                if ready {
                    break;
                }
                if start.elapsed() >= timeout {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "session startup timed out",
                    ));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        self.reader.read(bytes)
    }
}
impl Write for Channel {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writer.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
impl Endpoints {
    pub fn prepare(&self, command: &mut Command, prefix: &str) {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let fds: Vec<i32> = self.0.iter().map(|f| raw(f) as i32).collect();
            command.env(format!("{prefix}_READ"), fds[0].to_string());
            command.env(format!("{prefix}_WRITE"), fds[1].to_string());
            if fds.len() == 6 {
                command.env(
                    "MULTIWFN_SESSION_GUI_FDS",
                    format!("{},{},{},{}", fds[2], fds[3], fds[4], fds[5]),
                );
            }
            unsafe {
                command.pre_exec(move || {
                    for fd in &fds {
                        if libc::fcntl(*fd, libc::F_SETFD, 0) < 0 {
                            return Err(io::Error::last_os_error());
                        }
                    }
                    Ok(())
                });
            }
        }
        #[cfg(windows)]
        {
            command
                .env(format!("{prefix}_BOOTSTRAP"), "2")
                .stdin(std::process::Stdio::piped());
        }
    }
    pub fn complete(&self, _child: &mut Child) -> io::Result<()> {
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::Foundation::{DuplicateHandle, DUPLICATE_SAME_ACCESS};
            use windows_sys::Win32::System::Threading::GetCurrentProcess;
            let mut message = Vec::new();
            for file in &self.0 {
                let mut remote = std::ptr::null_mut();
                if unsafe {
                    DuplicateHandle(
                        GetCurrentProcess(),
                        file.as_raw_handle(),
                        _child.as_raw_handle(),
                        &mut remote,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    )
                } == 0
                {
                    return Err(io::Error::last_os_error());
                }
                message.extend_from_slice(&(remote as usize as u64).to_le_bytes());
            }
            _child
                .stdin
                .take()
                .ok_or_else(|| io::Error::other("missing session bootstrap"))?
                .write_all(&message)?;
        }
        Ok(())
    }
}
