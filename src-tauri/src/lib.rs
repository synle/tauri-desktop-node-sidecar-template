// Prevents an additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(debug_assertions))]
use std::io::BufRead;
#[cfg(all(not(debug_assertions), target_os = "windows"))]
use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
#[cfg(not(debug_assertions))]
use std::process::Stdio;
use std::sync::Mutex;
#[cfg(not(debug_assertions))]
use std::time::{Duration, Instant};
use std::time::Duration as StdDuration;
use std::time::Instant as StdInstant;
use tauri::Manager;

#[cfg(all(not(debug_assertions), target_os = "windows"))]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Holds the sidecar Node.js child process and the port it is listening on.
struct SidecarState {
    child: Mutex<Option<Child>>,
    port: u16,
}

/// Returns the app version baked in at compile time by `build.rs`.
#[tauri::command]
fn get_app_version() -> &'static str {
    env!("APP_VERSION")
}

/// Returns the port the sidecar Express server is listening on.
/// Returns `0` in dev mode (frontend uses Vite proxy to port 3001).
#[tauri::command]
fn get_sidecar_port(state: tauri::State<SidecarState>) -> u16 {
    state.port
}

/// Searches for a working `node` binary outside the default PATH.
///
/// GUI apps on macOS/Linux don't inherit shell rc files, so version managers
/// (fnm, nvm, volta, mise, n, asdf, Homebrew, nodenv) are invisible. Returns
/// the first path where `node --version` succeeds.
#[cfg(not(debug_assertions))]
fn find_system_node() -> Option<String> {
    let mut probe = Command::new("node");
    probe.arg("--version").stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(target_os = "windows")]
    probe.creation_flags(CREATE_NO_WINDOW);
    if probe.status().map(|s| s.success()).unwrap_or(false) {
        return Some("node".to_string());
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let candidates: Vec<std::path::PathBuf> = vec![
        std::path::PathBuf::from(&home).join(".local/share/fnm/aliases/default/bin/node"),
        std::path::PathBuf::from(&home)
            .join("Library/Application Support/fnm/aliases/default/bin/node"),
        std::path::PathBuf::from(&home).join(".volta/bin/node"),
        std::path::PathBuf::from(&home).join(".local/share/mise/shims/node"),
        std::path::PathBuf::from("/usr/local/bin/node"),
        std::path::PathBuf::from(&home).join(".asdf/shims/node"),
        std::path::PathBuf::from(&home).join(".nodenv/shims/node"),
        std::path::PathBuf::from("/opt/homebrew/bin/node"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            let mut probe = Command::new(&candidate);
            probe.arg("--version").stdout(Stdio::null()).stderr(Stdio::null());
            #[cfg(target_os = "windows")]
            probe.creation_flags(CREATE_NO_WINDOW);
            if probe.status().map(|s| s.success()).unwrap_or(false) {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Spawns the bundled Node.js sidecar in production builds and waits for it
/// to print `__SIDECAR_PORT__=<n>` on stdout.
#[cfg(not(debug_assertions))]
fn spawn_sidecar(app: &tauri::App) -> Result<SidecarState, Box<dyn std::error::Error>> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("failed to resolve resource dir: {e}"))?;
    let server_js = resource_dir.join("resources").join("server.cjs");

    let node_cmd = find_system_node().unwrap_or_else(|| "node".to_string());
    println!("sidecar: spawning {} {}", node_cmd, server_js.display());

    let mut spawn_cmd = Command::new(&node_cmd);
    spawn_cmd
        .arg(&server_js)
        .env("SIDECAR_PORT", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    #[cfg(target_os = "windows")]
    spawn_cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = spawn_cmd.spawn().map_err(|e| format!("failed to spawn sidecar: {e}"))?;

    let stdout = child.stdout.take().ok_or("failed to capture sidecar stdout")?;
    let reader = std::io::BufReader::new(stdout);
    let start = Instant::now();
    let timeout = Duration::from_secs(15);
    let mut port: u16 = 0;
    for line in reader.lines() {
        if start.elapsed() > timeout {
            let _ = child.kill();
            return Err("sidecar startup timed out (15s)".into());
        }
        match line {
            Ok(text) => {
                println!("sidecar: {text}");
                if let Some(port_str) = text.strip_prefix("__SIDECAR_PORT__=") {
                    port = port_str.trim().parse().map_err(|_| format!("invalid port: {port_str}"))?;
                    break;
                }
            }
            Err(e) => {
                let _ = child.kill();
                return Err(format!("failed to read sidecar stdout: {e}").into());
            }
        }
    }
    if port == 0 {
        let _ = child.kill();
        return Err("sidecar did not report a port".into());
    }
    println!("sidecar: ready on port {port}");
    Ok(SidecarState { child: Mutex::new(Some(child)), port })
}

/// Gracefully kills the sidecar child process.
fn kill_sidecar(state: &SidecarState) {
    if let Ok(mut guard) = state.child.lock() {
        if let Some(mut child) = guard.take() {
            // Drop stdin to signal the sidecar to shut down.
            drop(child.stdin.take());

            let start = StdInstant::now();
            let timeout = StdDuration::from_secs(3);
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => return,
                    Ok(None) if start.elapsed() > timeout => {
                        let _ = child.kill();
                        return;
                    }
                    _ => std::thread::sleep(StdDuration::from_millis(100)),
                }
            }
        }
    }
}

/// Tauri application entry point.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            // Dev mode: the sidecar is started by `npm run dev` on port 3001
            // and the Vite proxy forwards `/api/*`. Skip spawning here.
            #[cfg(debug_assertions)]
            let state = SidecarState { child: Mutex::new(None), port: 0 };

            #[cfg(not(debug_assertions))]
            let state = spawn_sidecar(_app).map_err(|e| {
                eprintln!("Fatal: {e}");
                e
            })?;

            _app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_app_version, get_sidecar_port])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| match event {
        tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
            if let Some(state) = app_handle.try_state::<SidecarState>() {
                kill_sidecar(&state);
            }
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_version_is_non_empty() {
        assert!(!get_app_version().is_empty());
    }
}
