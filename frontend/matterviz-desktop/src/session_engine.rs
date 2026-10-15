//! One engine owned by the desktop. The native menu bridge acknowledges
//! consumed input at a main-menu boundary, independently of human-readable logs.
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use crate::control_transport::ControlTransportConfig;
use crate::service::AppConfig;
use crate::session_pipe::{self, Channel};
use crate::transport::TransportConfig;

type Hook = Box<dyn Fn() -> Result<(), String> + Send + Sync>;
pub struct Hooks {
    pub leave: Hook,
    pub backend_lock: Arc<Mutex<()>>,
}
struct Engine {
    child: Child,
    channel: Channel,
    consumed: u64,
    in_gui: bool,
    file: PathBuf,
}
impl Drop for Engine {
    fn drop(&mut self) {
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
pub struct SessionEngine {
    engine: Mutex<Engine>,
    hooks: OnceLock<Hooks>,
    pub pid: u32,
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
        let (channel, mut endpoints) = Channel::pair().map_err(err)?;
        let (volume, native_volume) = session_pipe::pipe().map_err(err)?;
        let (native_ack, ack) = session_pipe::pipe().map_err(err)?;
        let (native_request, control_write) = session_pipe::pipe().map_err(err)?;
        let (control_read, native_response) = session_pipe::pipe().map_err(err)?;
        endpoints
            .0
            .extend([native_volume, native_ack, native_request, native_response]);
        let log_root = std::env::temp_dir().join("multiwfn-session-logs");
        fs::create_dir_all(&log_root).map_err(err)?;
        let log = File::create(log_root.join(format!("desktop-{}.log", std::process::id())))
            .map_err(err)?;
        let mut command = Command::new(executable);
        command
            .arg(&file)
            .args(["-nt", "2"])
            .current_dir(executable.parent().unwrap())
            .env("GFORTRAN_UNBUFFERED_PRECONNECTED", "y")
            .env("OMP_STACKSIZE", "64M")
            .stdin(Stdio::null())
            .stdout(log.try_clone().map_err(err)?)
            .stderr(log);
        endpoints.prepare(&mut command, "MULTIWFN_SESSION");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn().map_err(err)?;
        let pid = child.id();
        if let Err(error) = endpoints.complete(&mut child) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(err(error));
        }
        drop(endpoints);
        let mut engine = Engine {
            child,
            channel,
            consumed: 0,
            in_gui: false,
            file,
        };
        engine
            .channel
            .set_read_timeout(Some(Duration::from_secs(120)));
        let ready: Result<(), String> = (|| {
            let mut version = [0; 4];
            engine.channel.read_exact(&mut version).map_err(err)?;
            if &version != b"MWS2" {
                return Err("Unsupported native session protocol".into());
            }
            if receive(&mut engine)? != b'R' {
                return Err("Missing initial ready event".into());
            }
            Ok(())
        })();
        if let Err(error) = ready {
            let _ = engine.child.kill();
            let _ = engine.child.wait();
            return Err(format!("Engine startup: {error}"));
        }
        engine.channel.set_read_timeout(None);
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
                volume_read_pipe: session_pipe::release(volume),
                volume_ack_pipe: session_pipe::release(ack),
            }),
            multiwfn_pid: Some(u64::from(pid)),
        };
        let control = ControlTransportConfig {
            read_pipe: session_pipe::release(control_read),
            write_pipe: session_pipe::release(control_write),
        };
        Ok((
            Arc::new(Self {
                engine: Mutex::new(engine),
                hooks: OnceLock::new(),
                pid,
            }),
            config,
            control,
        ))
    }
    pub fn bind(&self, hooks: Hooks) {
        let _ = self.hooks.set(hooks);
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
    pub fn close(&self) -> Result<(), String> {
        if self.exited()? {
            return Ok(());
        }
        let hooks = self.hooks.get().ok_or("Session not attached")?;
        let _guard = hooks.backend_lock.lock().map_err(err)?;
        let mut engine = self.engine.lock().map_err(err)?;
        self.leave(&mut engine)?;
        engine.channel.write_all(b"Q").map_err(err)?;
        engine.child.wait().map_err(err)?;
        Ok(())
    }
    pub fn exited(&self) -> Result<bool, String> {
        Ok(self
            .engine
            .lock()
            .map_err(err)?
            .child
            .try_wait()
            .map_err(err)?
            .is_some())
    }
}
