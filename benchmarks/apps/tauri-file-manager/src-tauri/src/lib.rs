//! Tauri comparison shell. Window geometry and timing marks follow benchmarks/results/SCHEMA.md
//! (same stderr format as the Aster applications), so both can be measured with the same script.
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn epoch_us() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_micros()).unwrap_or(0)
}

fn timing_enabled() -> bool {
    std::env::var_os("ASTER_TIMING").is_some()
}

fn compare_enabled() -> bool {
    std::env::var_os("ASTER_COMPARE_GEOMETRY").is_some()
}

#[tauri::command]
fn report_first_frame() {
    eprintln!("FIRST_FRAME_US={}", epoch_us());
}

#[tauri::command]
fn report_geometry(json: String) {
    eprintln!("GEOMETRY_JSON={json}");
}

#[tauri::command]
fn report_selftest(json: String) {
    eprintln!("SELFTEST_JSON={json}");
}

pub fn run() {
    if timing_enabled() {
        eprintln!("MAIN_START_US={}", epoch_us());
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![report_first_frame, report_geometry, report_selftest])
        .setup(|app| {
            // Flags for the page. Only the page's timing and geometry hooks read them.
            let selftest = std::env::var_os("ASTER_COMPARE_SELFTEST").is_some();
            let script = format!(
                "window.__ASTER_TIMING__ = {}; window.__ASTER_COMPARE__ = {}; window.__ASTER_SELFTEST__ = {};",
                timing_enabled(),
                compare_enabled(),
                selftest
            );
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("File Manager")
                .inner_size(1100.0, 720.0)
                .min_inner_size(800.0, 500.0)
                .initialization_script(&script)
                .build()?;
            // Requirement F1: 1100 x 720 logical px for the content area (requirements.md section 1).
            // On macOS the window frame includes a 28 pt title bar, so the frame is 748 pt high and the
            // web view is 720 pt high. Measured: innerHeight 720 with this value; 692 with 720.
            // The Aster window has the same frame (748) and content (720) geometry.
            // ASTER_COMPARE_MIN_WINDOW=1 starts at the minimum size (requirement F1: 800 x 500 content).
            let (w, h) = if std::env::var_os("ASTER_COMPARE_MIN_WINDOW").is_some() {
                (800.0, 528.0)
            } else {
                (1100.0, 748.0)
            };
            window.set_size(tauri::LogicalSize::new(w, h))?;
            if timing_enabled() {
                eprintln!("WINDOW_READY_US={}", epoch_us());
                eprintln!("SCALE_FACTOR={}", window.scale_factor()?);
            }
            let _ = app.get_webview_window("main");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
