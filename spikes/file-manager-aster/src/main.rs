use aster_ui_model::{apply_patch, PatchOp};
use file_manager_aster::{create, spec::file_manager_spec};
use slint::ComponentHandle;
use std::time::{SystemTime, UNIX_EPOCH};

/// Wall-clock time in microseconds since the Unix epoch. Parent and child use the same clock,
/// so the timing script can subtract spawn time from child-reported times.
fn epoch_us() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0)
}

fn timing_enabled() -> bool {
    std::env::var_os("ASTER_TIMING").is_some()
}

/// Usage: file-manager-aster [--patch <file.json>] [--backend slint|text]
/// `--backend text` prints the model as a text tree instead of opening a window (experiment 05).
/// The patch is a JSON array of structured operations (03 section 6). It is applied to the
/// model before the window is built, so the displayed window reflects the change.
fn main() {
    let timing = timing_enabled();
    if timing {
        eprintln!("MAIN_START_US={}", epoch_us());
    }
    let mut spec = file_manager_spec();
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--patch") {
        let path = match args.get(pos + 1) {
            Some(p) => p,
            None => {
                eprintln!("--patch needs a file argument");
                std::process::exit(2);
            }
        };
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("cannot read {path}: {e}");
            std::process::exit(2);
        });
        let ops: Vec<PatchOp> = serde_json::from_str(&text).unwrap_or_else(|e| {
            eprintln!("invalid patch JSON: {e}");
            std::process::exit(2);
        });
        spec = apply_patch(&spec, &ops).unwrap_or_else(|e| {
            eprintln!("patch rejected: {e:?}");
            std::process::exit(1);
        });
    }
    let backend = args
        .iter()
        .position(|a| a == "--backend")
        .and_then(|pos| args.get(pos + 1).cloned())
        .unwrap_or_else(|| "slint".to_string());
    if backend == "text" {
        print!("{}", aster_backend_text::render(&spec));
        return;
    }
    if backend != "slint" {
        eprintln!("unknown backend: {backend}");
        std::process::exit(2);
    }
    let (session, _app) = match create(&spec) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to start: {e}");
            std::process::exit(1);
        }
    };
    if timing {
        eprintln!("WINDOW_READY_US={}", epoch_us());
        let weak_instance = session.instance().as_weak();
        let mut reported = false;
        let notifier = session
            .instance()
            .window()
            .set_rendering_notifier(move |state, _| {
                if !reported && matches!(state, slint::RenderingState::AfterRendering) {
                    reported = true;
                    // Read the scale factor here, after the window is shown. Before that it can report 1.0.
                    if let Some(instance) = weak_instance.upgrade() {
                        eprintln!("SCALE_FACTOR={}", instance.window().scale_factor());
                    }
                    eprintln!("FIRST_FRAME_US={}", epoch_us());
                }
            });
        if let Err(e) = notifier {
            eprintln!("rendering notifier unavailable: {e}");
        }
    }
    if let Err(e) = session.instance().run() {
        eprintln!("event loop error: {e}");
        std::process::exit(1);
    }
}
