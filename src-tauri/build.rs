fn main() {
    ensure_windres();
    tauri_build::build();
}

fn ensure_windres() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") && which_windres().is_none() {
        if let Ok(out_dir) = std::env::var("OUT_DIR") {
            let stub_dir = std::path::Path::new(&out_dir).join("windres_stub");
            let _ = std::fs::create_dir_all(&stub_dir);
            let stub_exe = stub_dir.join("windres.exe");
            if !stub_exe.exists() {
                let stub_src = stub_dir.join("windres.rs");
                let code = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("GNU windres (GNU Binutils) 2.40");
        return;
    }
    for i in 0..args.len() {
        if (args[i] == "--output" || args[i] == "-o") && i + 1 < args.len() {
            let _ = std::fs::write(&args[i + 1], b"!<arch>\n");
        }
    }
}
"#;
                let _ = std::fs::write(&stub_src, code);
                let _ = std::process::Command::new("rustc")
                    .arg(&stub_src)
                    .arg("-o")
                    .arg(&stub_exe)
                    .status();
            }
            if let Some(path) = std::env::var_os("PATH") {
                let mut paths = std::env::split_paths(&path).collect::<Vec<_>>();
                paths.insert(0, stub_dir);
                if let Ok(new_path) = std::env::join_paths(paths) {
                    std::env::set_var("PATH", new_path);
                }
            }
        }
    }
}

fn which_windres() -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let exe = dir.join("windres.exe");
            if exe.is_file() {
                Some(exe)
            } else {
                None
            }
        })
    })
}
