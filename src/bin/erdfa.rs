//! eRDFa CLI — serve, build, convert
//!
//! Usage:
//!   erdfa serve [--port 8080] [--dir _site]
//!   erdfa build [--out _site]
//!   erdfa convert <url> [--out _site]

use std::env;
use std::fs;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: erdfa <serve|build|convert> [options]");
        eprintln!("  serve  [--port 8080] [--dir _site]  Start local dev server");
        eprintln!("  build  [--out _site]                 Build static site + WASM");
        eprintln!("  convert <url> [--out _site]          Bake paste URL into static site");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "serve" => cmd_serve(&args[2..]),
        "build" => cmd_build(&args[2..]),
        "convert" => cmd_convert(&args[2..]),
        other => { eprintln!("Unknown command: {other}"); std::process::exit(1); }
    }
}

// --- serve ---

fn cmd_serve(args: &[String]) {
    let mut port: u16 = 8080;
    let mut dir = PathBuf::from("_site");
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => { i += 1; port = args[i].parse().expect("bad port"); }
            "--dir"  => { i += 1; dir = PathBuf::from(&args[i]); }
            _ => {}
        }
        i += 1;
    }
    if !dir.exists() {
        eprintln!("Directory {} not found. Run `erdfa build` first.", dir.display());
        std::process::exit(1);
    }
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr).expect("bind failed");
    eprintln!("🌐 Serving {} at http://{addr}", dir.display());
    for stream in listener.incoming().flatten() {
        let dir = dir.clone();
        std::thread::spawn(move || handle_request(stream, &dir));
    }
}

fn handle_request(mut stream: std::net::TcpStream, root: &Path) {
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() { return; }
    // Drain headers
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line.trim().is_empty() { break; }
    }
    let path = request_line.split_whitespace().nth(1).unwrap_or("/");
    let path = if path == "/" { "/index.html" } else { path };
    // Strip query string
    let path = path.split('?').next().unwrap_or(path);
    let file_path = root.join(&path[1..]); // strip leading /
    let (status, body, ctype) = if file_path.exists() && file_path.is_file() {
        let body = fs::read(&file_path).unwrap_or_default();
        let ctype = mime_type(&file_path);
        ("200 OK", body, ctype)
    } else {
        ("404 Not Found", b"404 Not Found".to_vec(), "text/plain")
    };
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&body);
}

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js")   => "application/javascript",
        Some("wasm") => "application/wasm",
        Some("css")  => "text/css",
        Some("json") => "application/json",
        Some("png")  => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("svg")  => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

// --- build ---

fn cmd_build(args: &[String]) {
    let mut out = PathBuf::from("_site");
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--out" { i += 1; out = PathBuf::from(&args[i]); }
        i += 1;
    }
    let root = find_repo_root();
    let wasm_dir = root.join("wasm");

    // 1. Build WASM
    eprintln!("🦀 Building WASM...");
    let status = Command::new("wasm-pack")
        .args(["build", "--target", "web", "--release"])
        .current_dir(&wasm_dir)
        .status()
        .expect("wasm-pack not found — install: cargo install wasm-pack");
    if !status.success() { eprintln!("wasm-pack failed"); std::process::exit(1); }

    // 2. Assemble _site
    eprintln!("📦 Assembling site in {}...", out.display());
    fs::create_dir_all(&out).unwrap();
    let pkg_src = wasm_dir.join("pkg");
    let pkg_dst = out.join("pkg");
    copy_dir(&pkg_src, &pkg_dst);

    // Copy index.html
    let index_src = wasm_dir.join("index.html");
    if index_src.exists() {
        fs::copy(&index_src, out.join("index.html")).unwrap();
    }

    // Copy docs/spec if present
    for dir_name in &["docs", "spec"] {
        let src = root.join(dir_name);
        if src.exists() { copy_dir(&src, &out.join(dir_name)); }
    }

    eprintln!("✅ Site built in {}", out.display());
}

// --- convert ---

fn cmd_convert(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: erdfa convert <url> [--out _site]");
        std::process::exit(1);
    }
    let url = &args[0];
    let mut out = PathBuf::from("_site");
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--out" { i += 1; out = PathBuf::from(&args[i]); }
        i += 1;
    }

    // Build first
    cmd_build(&["--out".into(), out.to_string_lossy().into()]);

    // Inject the URL as auto-execute param into index.html
    let index_path = out.join("index.html");
    let html = fs::read_to_string(&index_path).expect("index.html not found after build");

    // Parse the paste URL's query string and bake it in
    let inject = format!(
        r#"<script>if(!location.search)location.replace(location.pathname+"?{}");</script>"#,
        url.trim_start_matches('?')
    );
    let html = html.replacen("<head>", &format!("<head>\n{inject}"), 1);
    fs::write(&index_path, html).unwrap();
    eprintln!("✅ Custom site in {} — auto-loads: {url}", out.display());
}

// --- helpers ---

fn find_repo_root() -> PathBuf {
    let mut dir = env::current_dir().unwrap();
    loop {
        if dir.join("wasm").join("Cargo.toml").exists() { return dir; }
        if dir.join("Cargo.toml").exists() && dir.join("wasm").exists() { return dir; }
        if !dir.pop() { break; }
    }
    // Fallback: current dir
    env::current_dir().unwrap()
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap().flatten() {
        let ty = entry.file_type().unwrap();
        let dest = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), dest).unwrap();
        }
    }
}
