//! Real GUI engine lifecycle, without the retired embedded workflow backend.
#![cfg(any(unix, windows))]
use matterviz_volume_e2e::{service::HttpService, session_engine::SessionEngine};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;
use url::Url;

#[test]
#[ignore = "requires built native GUI engine and frontend; XYZ I/O only"]
fn desktop_session_opens_and_closes_without_workflows() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let base = PathBuf::from(
        std::env::var("MULTIWFN_SESSION_CHECK_DIR").expect("new check directory required"),
    );
    let build = std::env::var_os("MULTIWFN_SESSION_BUILD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo.join("build"));
    fs::create_dir(&base).unwrap();
    let input = base.join("single atom.xyz");
    fs::write(&input, "1\nGUI lifecycle fixture\nHe 0.0 0.0 0.0\n").unwrap();
    let resources = if build.join("resources").is_dir() {
        build.join("resources")
    } else {
        build.join("../Resources")
    };
    let (owner, config, control) = SessionEngine::launch(
        &build.join(format!(
            "Multiwfn_MatterVizGUI{}",
            std::env::consts::EXE_SUFFIX
        )),
        &input,
        resources.join("frontend/matterviz-viewer/dist"),
    )
    .unwrap();
    let service = HttpService::start_with_control(config, Some(control)).unwrap();
    service.attach_session(owner.clone()).unwrap();
    let url = Url::parse(service.url()).unwrap();
    let cap = url
        .query_pairs()
        .find(|(k, _)| k == "cap")
        .unwrap()
        .1
        .to_string();
    for (path, status) in [
        ("/session/manifest.json".to_string(), "200"),
        (format!("/api/workflows/status?cap={cap}"), "404"),
    ] {
        let mut stream = TcpStream::connect(("127.0.0.1", url.port().unwrap())).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            url.authority()
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(
            response.starts_with(&format!("HTTP/1.1 {status}")),
            "{response}"
        );
    }
    service.signal_return().unwrap();
    assert!(owner.exited().unwrap());
    assert_eq!(service.termination_exit_code(), 0);
    service.shutdown();
    service.join();
    println!("PASS: native GUI bootstrap, no workflow API, clean engine exit");
}
