use aster_ui_model::{apply_patch, PatchOp};
use file_manager_aster::{create, spec::file_manager_spec};
use slint::ComponentHandle;

/// Usage: file-manager-aster [--patch <file.json>] [--backend slint|text]
/// `--backend text` prints the model as a text tree instead of opening a window (experiment 05).
/// The patch is a JSON array of structured operations (03 section 6). It is applied to the
/// model before the window is built, so the displayed window reflects the change.
fn main() {
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
    if let Err(e) = session.instance().run() {
        eprintln!("event loop error: {e}");
        std::process::exit(1);
    }
}
