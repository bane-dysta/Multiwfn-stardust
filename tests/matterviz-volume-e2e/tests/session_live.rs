//! Real native process and real C++ runner, using menu state and XYZ I/O only.
//! No electronic-structure calculation or wavefunction analysis is performed.
#![cfg(any(unix, windows))]
use matterviz_volume_e2e::{
    service::HttpService, session_engine::SessionEngine, workflows::WorkflowManager,
};
use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use url::Url;

fn api(base: &str, action: &str, body: Option<Value>) -> Value {
    let url = Url::parse(base).unwrap();
    let cap = url
        .query_pairs()
        .find(|(k, _)| k == "cap")
        .unwrap()
        .1
        .to_string();
    let mut stream = TcpStream::connect(("127.0.0.1", url.port().unwrap())).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    let data = body.as_ref().map(|v| v.to_string()).unwrap_or_default();
    let method = if body.is_some() { "POST" } else { "GET" };
    write!(stream,"{method} /api/workflows/{action}?cap={cap} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{data}",url.authority(),data.len()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let (_, body) = response.split_once("\r\n\r\n").unwrap();
    serde_json::from_str(body).unwrap_or_else(|_| panic!("{response}"))
}
fn run(base: &str, script: &str) -> Value {
    let started = api(
        base,
        "run",
        Some(json!({"script":script,"cores":2,"variables":[],"dryRun":false})),
    );
    assert!(started.get("job").is_some(), "{started}");
    finish(base)
}
fn finish(base: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let status = api(base, "status", None);
        if !matches!(
            status["job"]["state"].as_str(),
            Some("running" | "cancelling")
        ) {
            return status;
        }
        assert!(Instant::now() < deadline, "timed out: {status}");
        thread::sleep(Duration::from_millis(50));
    }
}
fn logs(directory: &Path) -> String {
    fs::read_dir(directory)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "out"))
        .map(|e| fs::read_to_string(e.path()).unwrap())
        .collect()
}

#[test]
#[ignore = "requires built native GUI engine, runner and frontend; no scientific calculation"]
fn persistent_process_preserves_memory_and_gui_transport() {
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
    let base = base.canonicalize().unwrap();
    let input = base.join("single atom.xyz");
    fs::write(&input, "1\nSession memory fixture\nHe 0.0 0.0 0.0\n").unwrap();
    let library = base.join("scripts");
    fs::create_dir(&library).unwrap();
    fs::write(library.join("set.bw"), "%raw\n1000\n5\n0.125\nend\n").unwrap();
    fs::write(library.join("crlf.bw"), "%raw\r\n1000\r\n0\r\nend\r\n").unwrap();
    fs::write(
        library.join("large.bw"),
        format!("%raw\n{}1000\n0\nend\n", "\n".repeat(96 * 1024)),
    )
    .unwrap();
    fs::write(library.join("read.bw"),"%raw\n1000\n0\nxyz\nretained.xyz\n%grep\nmatch \"1.25000000E-01\" -> retained-state.txt\nend\n").unwrap();
    fs::write(
        library.join("bad-exit.bw"),
        "%raw\nq\nend\n%command\ntouch must-not-run\nend\n",
    )
    .unwrap();
    fs::write(library.join("cancel.bw"), "%command\ntouch entered\nsleep 1\ntouch finished-block\nend\n%command\ntouch must-not-run\nend\n").unwrap();
    #[cfg(windows)]
    {
        fs::write(
            library.join("bad-exit.bw"),
            "%raw\nq\nend\n%command\necho bad>must-not-run\nend\n",
        )
        .unwrap();
        fs::write(library.join("cancel.bw"), "%command\necho entered>entered\npowershell -NoProfile -Command Start-Sleep -Seconds 2\necho finished>finished-block\nend\n%command\necho bad>must-not-run\nend\n").unwrap();
    }
    fs::write(library.join("module.bwc"), "[memory]\n%process\nset\nend\n#>>> BANEWFN_INLINE_CONF_BEGIN memory\n# [main]\n# 1000\n# [set]\n# 5\n# 0.125\n# [quit]\n# q\n#<<< BANEWFN_INLINE_CONF_END memory\n").unwrap();
    // Unlike an inline .bwc, a .bw must open the staged external conf directory.
    // Windows canonical paths use \\?\; handing those to the C++ config loader
    // used to fail when it appended a forward slash before the module filename.
    fs::create_dir(library.join("conf")).unwrap();
    fs::write(
        library.join("conf/native_io.conf"),
        "[main]\nxyz\n[export]\nmodule.xyz\n[quit]\nq\n",
    )
    .unwrap();
    fs::write(
        library.join("external.bw"),
        "[native_io]\n%process\nexport\nend\n",
    )
    .unwrap();
    fs::write(
        library.join("reload.bw"),
        "wfn_rebase=\n%raw\nxyz\nreloaded.xyz\nend\n",
    )
    .unwrap();
    fs::write(library.join("memory.cub"), "Memory fixture\nEight stored values\n1 0 0 0\n2 1 0 0\n2 0 1 0\n2 0 0 1\n2 2 0 0 0\n1 2 3 4 5 6\n7 8\n").unwrap();
    fs::write(library.join("load-grid.bw"), "%raw\nr\nmemory.cub\nend\n").unwrap();
    fs::write(library.join("read-grid.bw"), "%raw\n13\n1\n-1\nend\n").unwrap();
    let resources = if build.join("resources").is_dir() {
        build.join("resources")
    } else {
        build.join("../Resources")
    };
    let suffix = std::env::consts::EXE_SUFFIX;
    std::env::set_var(
        "MULTIWFN_WORKFLOW_RUNNER",
        resources.join(format!("tools/multiwfn-workflow{suffix}")),
    );
    std::env::set_var("MULTIWFN_WORKFLOW_LIBRARY", &library);
    std::env::set_var("MULTIWFN_WORKFLOW_CONF", resources.join("workflows/conf"));
    let (owner, config, control) = SessionEngine::launch(
        &build.join(format!("Multiwfn_MatterVizGUI{suffix}")),
        &input,
        resources.join("frontend/matterviz-viewer/dist"),
    )
    .unwrap();
    let service = HttpService::start_with_control(config, Some(control)).unwrap();
    service.attach_session(owner.clone()).unwrap();
    println!("Live session PID: {}", owner.pid);
    // Also exercise the legacy engine-first manager: the previous package test
    // wrote its own POSIX-style rc file and missed canonicalized GUI selections.
    let batch = WorkflowManager::new(Some(u64::from(owner.pid)), Some(&input));
    fs::create_dir(base.join("batch results")).unwrap();
    batch
        .select("results", &base.join("batch results"))
        .unwrap();
    batch
        .start(&json!({"script":"external.bw","cores":1,"variables":[],"dryRun":false}))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let status = batch.status();
        if status["job"]["state"] != "running" {
            assert_eq!(status["job"]["state"], "completed", "{status}");
            assert!(status["session"].is_null());
            assert!(fs::read_to_string(
                Path::new(status["job"]["directory"].as_str().unwrap()).join("module.xyz")
            )
            .unwrap()
            .contains("He"));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "batch startup timed out: {status}"
        );
        thread::sleep(Duration::from_millis(50));
    }
    batch.shutdown();
    let first = run(service.url(), "set.bw");
    assert_eq!(first["job"]["state"], "completed", "{first}");
    assert_eq!(first["session"]["pid"], owner.pid);
    // A reload from disk is now impossible; later work must use retained memory.
    fs::rename(&input, base.join("moved-input.xyz")).unwrap();
    let second = run(service.url(), "read.bw");
    assert_eq!(second["job"]["state"], "completed", "{second}");
    assert_eq!(second["session"]["pid"], owner.pid);
    let directory = PathBuf::from(second["job"]["directory"].as_str().unwrap());
    assert!(logs(&directory).contains("1.25000000E-01"));
    assert!(directory.join("retained.xyz").is_file());
    assert!(directory.join("retained-state.txt").is_file());
    let rejected = run(service.url(), "bad-exit.bw");
    assert_eq!(rejected["job"]["state"], "failed", "{rejected}");
    assert!(!Path::new(rejected["job"]["directory"].as_str().unwrap())
        .join("must-not-run")
        .exists());
    let third = run(service.url(), "read.bw");
    assert_eq!(third["job"]["state"], "completed", "{third}");
    assert_eq!(third["session"]["pid"], owner.pid);
    for name in ["crlf.bw", "large.bw"] {
        let checked = run(service.url(), name);
        assert_eq!(checked["job"]["state"], "completed", "{name}: {checked}");
        assert_eq!(checked["session"]["pid"], owner.pid);
    }
    let module = run(service.url(), "module.bwc");
    assert_eq!(module["job"]["state"], "completed", "{module}");
    let external = run(service.url(), "external.bw");
    assert_eq!(external["job"]["state"], "completed", "{external}");
    assert_eq!(external["session"]["pid"], owner.pid);
    assert!(fs::read_to_string(
        Path::new(external["job"]["directory"].as_str().unwrap()).join("module.xyz")
    )
    .unwrap()
    .contains("He"));
    let started = api(
        service.url(),
        "run",
        Some(json!({"script":"cancel.bw","cores":2,"variables":[],"dryRun":false})),
    );
    let directory = PathBuf::from(started["job"]["directory"].as_str().unwrap());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !directory.join("entered").exists() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    api(service.url(), "cancel", Some(json!({})));
    let cancelled = finish(service.url());
    assert_eq!(cancelled["job"]["state"], "cancelled", "{cancelled}");
    assert!(directory.join("finished-block").exists());
    assert!(!directory.join("must-not-run").exists());
    // Choosing a new startup file explicitly replaces memory in the same PID.
    fs::write(&input, "1\nReplacement\nNe 0.0 0.0 0.0\n").unwrap();
    owner.load_file(&input).unwrap();
    // The same-path rebase must really reload, not just compare path strings.
    fs::write(&input, "1\nExplicit rebase replacement\nAr 0.0 0.0 0.0\n").unwrap();
    let reloaded = run(service.url(), "reload.bw");
    assert_eq!(reloaded["job"]["state"], "completed", "{reloaded}");
    assert!(fs::read_to_string(
        Path::new(reloaded["job"]["directory"].as_str().unwrap()).join("reloaded.xyz")
    )
    .unwrap()
    .contains("Ar"));
    assert_eq!(reloaded["session"]["pid"], owner.pid);
    // Exercise native reload and output staging with a UTF-8 startup filename.
    let unicode_input = base.join("启动 文件.xyz");
    fs::write(&unicode_input, "1\nUnicode path\nAr 0.0 0.0 0.0\n").unwrap();
    owner.load_file(&unicode_input).unwrap();
    let unicode_run = run(service.url(), "reload.bw");
    assert_eq!(unicode_run["job"]["state"], "completed", "{unicode_run}");
    assert_eq!(unicode_run["session"]["pid"], owner.pid);
    let grid_loaded = run(service.url(), "load-grid.bw");
    assert_eq!(grid_loaded["job"]["state"], "completed", "{grid_loaded}");
    let loaded_path = PathBuf::from(grid_loaded["wavefunction"].as_str().unwrap());
    assert_eq!(
        grid_loaded["results"],
        json!(loaded_path.parent().unwrap().join("memory_workflows"))
    );
    fs::rename(&loaded_path, loaded_path.with_extension("moved-cub")).unwrap();
    let grid_read = run(service.url(), "read-grid.bw");
    assert_eq!(grid_read["job"]["state"], "completed", "{grid_read}");
    let points = fs::read_to_string(
        Path::new(grid_read["job"]["directory"].as_str().unwrap()).join("output.txt"),
    )
    .unwrap();
    let mut values: Vec<f64> = points
        .lines()
        .map(|line| line.split_whitespace().last().unwrap().parse().unwrap())
        .collect();
    values.sort_by(f64::total_cmp);
    assert_eq!(values, vec![1., 2., 3., 4., 5., 6., 7., 8.]);
    assert_eq!(grid_read["session"]["pid"], owner.pid);
    fs::write(base.join("session-validation.json"),serde_json::to_vec_pretty(&json!({"pid":owner.pid,"first":first,"second":second,"rejectedExit":rejected,"continued":third,"inlineModule":module,"cancelled":cancelled,"reloaded":reloaded,"gridLoaded":grid_loaded,"gridRead":grid_read,"retainedGridValues":values})).unwrap()).unwrap();
    service.signal_return().unwrap();
    assert_eq!(service.termination_exit_code(), 0);
    service.shutdown();
    service.join();
    assert!(owner.exited().unwrap());
}
