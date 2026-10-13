//! One Linux engine owned by the desktop. The native menu bridge acknowledges
//! consumed input at a main-menu boundary, independently of human-readable logs.
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, IntoRawFd};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::control_transport::ControlTransportConfig;
use crate::service::AppConfig;
use crate::transport::TransportConfig;

type Hook = Box<dyn Fn() -> Result<(), String> + Send + Sync>;
pub struct Hooks {
    pub leave: Hook,
    pub prepare_refresh: Hook,
    pub refresh: Hook,
    pub backend_lock: Arc<Mutex<()>>,
}
struct Engine {
    child: Child,
    input: ChildStdin,
    channel: UnixStream,
    consumed: u64,
    in_gui: bool,
    file: PathBuf,
}
pub struct SessionEngine {
    engine: Mutex<Engine>,
    hooks: OnceLock<Hooks>,
    pub pid: u32,
    pub revision: AtomicU64,
    pub busy: AtomicBool,
}
fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}
pub fn read_string(stream: &mut impl Read, limit: usize) -> io::Result<String> {
    let mut size = [0; 4];
    stream.read_exact(&mut size)?;
    let size = u32::from_le_bytes(size) as usize;
    if size > limit {
        return Err(io::Error::other("session field too large"));
    }
    let mut bytes = vec![0; size];
    stream.read_exact(&mut bytes)?;
    let text = String::from_utf8(bytes).map_err(io::Error::other)?;
    if text.contains('\0') {
        return Err(io::Error::other("NUL in session field"));
    }
    Ok(text)
}
fn write_string(stream: &mut impl Write, text: &str) -> io::Result<()> {
    stream.write_all(&(text.len() as u32).to_le_bytes())?;
    stream.write_all(text.as_bytes())
}
fn receive(engine: &mut Engine) -> Result<u8, String> {
    let mut kind = [0];
    engine.channel.read_exact(&mut kind).map_err(err)?;
    if !b"RDE".contains(&kind[0]) {
        return Err("Invalid native session event".into());
    }
    let mut position = [0; 8];
    engine.channel.read_exact(&mut position).map_err(err)?;
    engine.consumed = u64::from_le_bytes(position);
    let file = read_string(&mut engine.channel, 32768).map_err(err)?;
    if !file.is_empty() {
        engine.file = PathBuf::from(file);
    }
    Ok(kind[0])
}
impl SessionEngine {
    pub fn launch(
        executable: &Path,
        file: &Path,
        frontend: PathBuf,
    ) -> Result<(Arc<Self>, AppConfig, ControlTransportConfig), String> {
        let file = file.canonicalize().map_err(err)?;
        let (channel, native_channel) = UnixStream::pair().map_err(err)?;
        let (volume, native_volume) = UnixStream::pair().map_err(err)?;
        let (ack, native_ack) = UnixStream::pair().map_err(err)?;
        let (control_write, native_request) = UnixStream::pair().map_err(err)?;
        let (control_read, native_response) = UnixStream::pair().map_err(err)?;
        let fds = [
            native_channel.as_raw_fd(),
            native_volume.as_raw_fd(),
            native_ack.as_raw_fd(),
            native_request.as_raw_fd(),
            native_response.as_raw_fd(),
        ];
        let log_root = executable
            .parent()
            .ok_or("Missing engine directory")?
            .join("session-logs");
        fs::create_dir_all(&log_root).map_err(err)?;
        let log = File::create(log_root.join(format!("desktop-{}.log", std::process::id())))
            .map_err(err)?;
        let mut command = Command::new(executable);
        command
            .arg(&file)
            .args(["-nt", "2"])
            .current_dir(executable.parent().unwrap())
            .env("MULTIWFN_SESSION_FD", fds[0].to_string())
            .env(
                "MULTIWFN_SESSION_GUI_FDS",
                format!("{},{},{},{}", fds[1], fds[2], fds[3], fds[4]),
            )
            .env("GFORTRAN_UNBUFFERED_PRECONNECTED", "y")
            .env("OMP_STACKSIZE", "64M")
            .stdin(Stdio::piped())
            .stdout(log.try_clone().map_err(err)?)
            .stderr(log);
        // Only the deliberately inherited endpoints survive exec.
        unsafe {
            command.pre_exec(move || {
                for fd in fds {
                    if libc::fcntl(fd, libc::F_SETFD, 0) < 0 {
                        return Err(io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }
        let mut child = command.spawn().map_err(err)?;
        let pid = child.id();
        drop((
            native_channel,
            native_volume,
            native_ack,
            native_request,
            native_response,
        ));
        let input = child.stdin.take().ok_or("Missing managed input")?;
        let mut engine = Engine {
            child,
            input,
            channel,
            consumed: 0,
            in_gui: false,
            file,
        };
        engine
            .channel
            .set_read_timeout(Some(Duration::from_secs(120)))
            .map_err(err)?;
        if let Err(error) = receive(&mut engine) {
            let _ = engine.child.kill();
            let _ = engine.child.wait();
            return Err(format!("Engine startup: {error}"));
        }
        engine.channel.set_read_timeout(None).map_err(err)?;
        engine.channel.write_all(b"G").map_err(err)?;
        engine.in_gui = true;
        let config = AppConfig {
            frontend,
            session: PathBuf::from(format!("managed-{pid}")),
            manifest: None,
            state: None,
            host: "127.0.0.1".into(),
            port: 0,
            transport: Some(TransportConfig {
                volume_read_pipe: volume.into_raw_fd() as u64,
                volume_ack_pipe: ack.into_raw_fd() as u64,
            }),
            multiwfn_pid: Some(u64::from(pid)),
        };
        let control = ControlTransportConfig {
            read_pipe: control_read.into_raw_fd() as u64,
            write_pipe: control_write.into_raw_fd() as u64,
        };
        Ok((
            Arc::new(Self {
                engine: Mutex::new(engine),
                hooks: OnceLock::new(),
                pid,
                revision: AtomicU64::new(0),
                busy: AtomicBool::new(false),
            }),
            config,
            control,
        ))
    }
    pub fn bind(&self, hooks: Hooks) {
        let _ = self.hooks.set(hooks);
    }
    pub fn file(&self) -> PathBuf {
        self.engine
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .file
            .clone()
    }
    pub fn load_file(&self, file: &Path) -> Result<(), String> {
        let file = file.canonicalize().map_err(err)?;
        if file.to_string_lossy().contains(['\n', '\r', '\0']) {
            return Err("Invalid input file path".into());
        }
        let hooks = self.hooks.get().ok_or("Session not attached")?;
        let _guard = hooks.backend_lock.lock().map_err(err)?;
        let directory = file.parent().ok_or("Missing input directory")?;
        let log = std::env::temp_dir().join(format!("multiwfn-session-load-{}.out", self.pid));
        self.busy.store(true, Ordering::Release);
        let result = self.block(&format!("r\n{}\n", file.display()), &log, directory, 2);
        let refresh = self.refresh();
        self.busy.store(false, Ordering::Release);
        result.and(refresh)
    }
    fn leave(&self, engine: &mut Engine) -> Result<(), String> {
        if engine.in_gui {
            (self.hooks.get().ok_or("Session not attached")?.leave)()?;
            if receive(engine)? != b'R' {
                return Err("Engine did not return to main menu".into());
            }
            engine.in_gui = false;
        }
        Ok(())
    }
    fn block(
        &self,
        commands: &str,
        log: &Path,
        directory: &Path,
        cores: u32,
    ) -> Result<(), String> {
        if commands.is_empty() || !commands.ends_with('\n') || commands.len() > 512 * 1024 {
            return Err("Invalid command block".into());
        }
        let mut engine = self.engine.lock().map_err(err)?;
        self.leave(&mut engine)?;
        let end = engine
            .consumed
            .checked_add(commands.len() as u64)
            .ok_or("Input counter overflow")?;
        engine.channel.write_all(b"B").map_err(err)?;
        engine.channel.write_all(&end.to_le_bytes()).map_err(err)?;
        engine
            .channel
            .write_all(&cores.to_le_bytes())
            .map_err(err)?;
        write_string(&mut engine.channel, &directory.to_string_lossy()).map_err(err)?;
        write_string(&mut engine.channel, &log.to_string_lossy()).map_err(err)?;
        engine.input.write_all(commands.as_bytes()).map_err(err)?;
        let kind = receive(&mut engine)?;
        if engine.consumed != end {
            return Err("Native input consumption mismatch".into());
        }
        if kind != b'D' {
            return Err("Script tried to exit the current Multiwfn session".into());
        }
        Ok(())
    }
    fn refresh(&self) -> Result<(), String> {
        let mut engine = self.engine.lock().map_err(err)?;
        if !engine.in_gui {
            (self
                .hooks
                .get()
                .ok_or("Session not attached")?
                .prepare_refresh)()?;
            engine.channel.write_all(b"G").map_err(err)?;
            (self.hooks.get().ok_or("Session not attached")?.refresh)()?;
            engine.in_gui = true;
            self.revision.fetch_add(1, Ordering::Release);
        }
        Ok(())
    }
    pub fn close(&self) -> Result<(), String> {
        let hooks = self.hooks.get().ok_or("Session not attached")?;
        let _guard = hooks.backend_lock.lock().map_err(err)?;
        let mut engine = self.engine.lock().map_err(err)?;
        if engine.child.try_wait().map_err(err)?.is_some() {
            return Ok(());
        }
        self.leave(&mut engine)?;
        engine.channel.write_all(b"Q").map_err(err)?;
        engine.child.wait().map_err(err)?;
        Ok(())
    }
    pub fn runner(
        self: &Arc<Self>,
        command: &mut Command,
        directory: PathBuf,
        cancel: Arc<AtomicBool>,
    ) -> Result<(UnixStream, JoinHandle<Result<(), String>>), String> {
        let (mut server, client) = UnixStream::pair().map_err(err)?;
        let fd = client.as_raw_fd();
        command.env("BANEWFN_SESSION_FD", "3");
        unsafe {
            command.pre_exec(move || {
                if libc::dup2(fd, 3) < 0 || libc::fcntl(3, libc::F_SETFD, 0) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let owner = self.clone();
        let worker = thread::spawn(move || {
            let hooks = owner.hooks.get().ok_or("Session not attached")?;
            let _guard = hooks.backend_lock.lock().map_err(err)?;
            owner.busy.store(true, Ordering::Release);
            let result = (|| {
                loop {
                    let mut kind = [0];
                    match server.read_exact(&mut kind) {
                        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                        Err(e) => return Err(err(e)),
                        Ok(()) => {}
                    }
                    if kind[0] == b'P' {
                        server
                            .write_all(&[u8::from(cancel.load(Ordering::Acquire))])
                            .map_err(err)?;
                        continue;
                    }
                    if kind[0] != b'B' && kind[0] != b'L' {
                        return Err("Invalid runner session request".into());
                    }
                    let reload = kind[0] == b'L';
                    let mut cores = [0; 4];
                    server.read_exact(&mut cores).map_err(err)?;
                    let cores = u32::from_le_bytes(cores);
                    if !(1..=1024).contains(&cores) {
                        return Err("Invalid session thread count".into());
                    }
                    let commands = read_string(&mut server, 512 * 1024).map_err(err)?;
                    let log = read_string(&mut server, 4096).map_err(err)?;
                    let input = read_string(&mut server, 32768).map_err(err)?;
                    let log = PathBuf::from(log);
                    if log.components().count() != 1 || log.file_name().is_none() {
                        return Err("Invalid block log path".into());
                    }
                    let mut outcome = Ok(());
                    if cancel.load(Ordering::Acquire) {
                        outcome = Err("Workflow cancelled".into());
                    }
                    if outcome.is_ok() && !input.is_empty() {
                        let requested = directory.join(&input);
                        if reload || requested != owner.file() {
                            let requested = requested.canonicalize().map_err(err)?;
                            if requested.to_string_lossy().contains(['\n', '\r']) {
                                return Err("Invalid input file path".into());
                            }
                            outcome = owner.block(
                                &format!("r\n{}\n", requested.display()),
                                &directory.join("session-load.out"),
                                &directory,
                                cores,
                            );
                        }
                    }
                    if outcome.is_ok() && !reload {
                        outcome = owner.block(&commands, &directory.join(log), &directory, cores);
                    }
                    server
                        .write_all(&[u8::from(outcome.is_err())])
                        .map_err(err)?;
                    write_string(&mut server, &owner.file().to_string_lossy()).map_err(err)?;
                    outcome?;
                }
                Ok(())
            })();
            let refresh = owner.refresh();
            owner.busy.store(false, Ordering::Release);
            result.and(refresh)
        });
        Ok((client, worker))
    }
}
