//! Headless layout check for the Aster file manager.
//!
//! Builds the window from the Aster model (optionally after a structured patch), lays it out at a
//! given content size, and prints every geometry diagnostic as JSON (one object per line), in the
//! field names of ASTER-IR-001 section 4. Exit code 0: no diagnostics; 1: diagnostics found;
//! 2: usage or setup error.
//!
//! This binary depends on the headless testing backend, so it is built only with the
//! `layout-probe` feature. The production application binary does not include it.
//!
//! Usage:
//!   aster-check-layout [--patch <file.json>] [--width <dp>] [--height <dp>]

use aster_backend_slint::probe::{check_geometry, GeometryDiagnostic};
use aster_ui_model::{apply_patch, PatchOp};
use file_manager_aster::{create, spec::file_manager_spec};
use slint::ComponentHandle;

struct Args {
    patch: Option<String>,
    width: f32,
    height: f32,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        patch: None,
        width: 1100.0,
        height: 720.0,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--patch" => args.patch = Some(it.next().ok_or("--patch needs a file")?),
            "--width" => {
                args.width = it
                    .next()
                    .ok_or("--width needs a value")?
                    .parse()
                    .map_err(|_| "bad --width")?
            }
            "--height" => {
                args.height = it
                    .next()
                    .ok_or("--height needs a value")?
                    .parse()
                    .map_err(|_| "bad --height")?
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(args)
}

fn severity(code: &str) -> &'static str {
    match code {
        // Structural failures: the layout cannot be trusted.
        "ELEMENT_NOT_FOUND" => "error",
        // Declared size differs from the laid-out size, or content leaves its parent.
        "GEOMETRY_MISMATCH" | "LAYOUT_OVERFLOW" => "error",
        _ => "warning",
    }
}

/// One JSON object in the ASTER-IR-001 diagnostic shape. `source` is null: the model has no
/// source map yet, so the node id is given instead.
fn to_json(d: &GeometryDiagnostic) -> String {
    serde_json::json!({
        "protocol_version": "0.1",
        "code": d.code,
        "severity": severity(d.code),
        "message": format!("{} on {}.{}: expected {}, actual {}, overflow {}",
                           d.code, d.node, d.property, d.expected, d.actual, d.overflow),
        "source": null,
        "component": d.node,
        "property": d.property,
        "actual": d.actual,
        "expected": d.expected,
        "overflow": d.overflow,
    })
    .to_string()
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\nusage: aster-check-layout [--patch <file.json>] [--width <dp>] [--height <dp>]");
            std::process::exit(2);
        }
    };
    i_slint_backend_testing::init_no_event_loop();

    let mut spec = file_manager_spec();
    if let Some(path) = &args.patch {
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
            std::process::exit(2);
        });
    }

    let (session, _app) = match create(&spec) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot create window: {e}");
            std::process::exit(2);
        }
    };
    if let Err(e) = session.instance().show() {
        eprintln!("cannot show window: {e}");
        std::process::exit(2);
    }
    session
        .instance()
        .window()
        .set_size(slint::LogicalSize::new(args.width, args.height));

    let diagnostics = check_geometry(session.instance(), &spec);
    for d in &diagnostics {
        println!("{}", to_json(d));
    }
    std::process::exit(if diagnostics.is_empty() { 0 } else { 1 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_is_an_error_with_the_ir_field_names() {
        let d = GeometryDiagnostic {
            code: "LAYOUT_OVERFLOW",
            node: "content".to_string(),
            property: "width",
            expected: 1100.0,
            actual: 1000.0,
            overflow: 140.0,
        };
        let v: serde_json::Value = serde_json::from_str(&to_json(&d)).unwrap();
        assert_eq!(v["protocol_version"], "0.1");
        assert_eq!(v["severity"], "error");
        assert_eq!(v["component"], "content");
        assert_eq!(v["overflow"], 140.0);
        assert!(v["source"].is_null());
    }

    #[test]
    fn unknown_codes_are_warnings_not_errors() {
        assert_eq!(severity("SOMETHING_ELSE"), "warning");
        assert_eq!(severity("ELEMENT_NOT_FOUND"), "error");
    }
}
