//! Experiment 03: a structured patch (sidebar 240dp -> 280dp) applied to the Aster model must be
//! visible in the real Slint layout, and the generated source must change only where expected.

use aster_backend_slint::to_slint_source;
use aster_ui_model::{apply_patch, property_text, PatchError, PatchOp};
use file_manager_aster::{create, spec::file_manager_spec};
use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;

fn patch_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("patches/exp03-sidebar-280.json")
}

fn width_of(instance: &impl ComponentHandle, id: &str) -> f32 {
    ElementHandle::find_by_element_id(instance, id)
        .next()
        .unwrap_or_else(|| panic!("element {id} not found"))
        .size()
        .width
}

#[test]
fn patched_sidebar_is_280_dp_in_the_real_layout() {
    i_slint_backend_testing::init_no_event_loop();

    let baseline = file_manager_spec();
    let ops: Vec<PatchOp> =
        serde_json::from_str(&std::fs::read_to_string(patch_path()).unwrap()).unwrap();
    let patched = apply_patch(&baseline, &ops).expect("patch applies");

    // Model level: the change is exactly one property.
    assert_eq!(
        property_text(&baseline, "sidebar", "width").as_deref(),
        Some("240dp")
    );
    assert_eq!(
        property_text(&patched, "sidebar", "width").as_deref(),
        Some("280dp")
    );

    // Source level: the generated Slint differs in exactly one line.
    let before = to_slint_source(&baseline).unwrap();
    let after = to_slint_source(&patched).unwrap();
    let changed: Vec<(&str, &str)> = before
        .lines()
        .zip(after.lines())
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(before.lines().count(), after.lines().count());
    assert_eq!(changed.len(), 1, "unexpected changed lines: {changed:?}");
    assert!(changed[0].0.contains("width: 240px"));
    assert!(changed[0].1.contains("width: 280px"));

    // Running window: sidebar is 280 dp and content takes the rest.
    let (session, _app) = create(&patched).expect("patched spec loads");
    session.instance().show().expect("window shows");
    let window_width = session.instance().window().size().width as f32
        / session.instance().window().scale_factor();
    let sidebar = width_of(session.instance(), "MainWindow::sidebar");
    let content = width_of(session.instance(), "MainWindow::content");
    println!("patched: window={window_width} sidebar={sidebar} content={content}");
    assert_eq!(sidebar, 280.0);
    assert_eq!(content, window_width - 280.0);
}

#[test]
fn stale_patch_is_rejected_and_the_window_is_not_built_from_it() {
    // A patch written against an old value (260dp) must not apply to the current model.
    let baseline = file_manager_spec();
    let stale = vec![PatchOp::SetProp {
        target: "sidebar".to_string(),
        name: "width".to_string(),
        value: "280dp".to_string(),
        expect: Some("260dp".to_string()),
    }];
    assert!(matches!(
        apply_patch(&baseline, &stale),
        Err(PatchError::Conflict { .. })
    ));
    assert_eq!(
        property_text(&baseline, "sidebar", "width").as_deref(),
        Some("240dp")
    );
}
