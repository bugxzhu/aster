//! Real layout geometry checks. The geometry comes from Slint's layout engine (headless testing
//! backend, feature `layout-probe`); it is compared against the declared Aster model.
//! This runs in its own process, so it is not a substitute for a visible window check.

use aster_backend_slint::probe::{check_geometry, GeometryDiagnostic};
use aster_ui_model::{apply_patch, find_mut, Kind, Length, Node, PatchOp, UiSpec, Widget};
use file_manager_aster::{create, spec::file_manager_spec};
use slint::ComponentHandle;

fn codes(d: &[GeometryDiagnostic]) -> Vec<&'static str> {
    d.iter().map(|x| x.code).collect()
}

#[test]
fn baseline_window_matches_declared_geometry() {
    i_slint_backend_testing::init_no_event_loop();
    let spec = file_manager_spec();
    let (session, _app) = create(&spec).expect("window created");
    session.instance().show().expect("shows");
    let diags = check_geometry(session.instance(), &spec);
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
}

#[test]
fn patched_window_matches_declared_geometry_at_280() {
    i_slint_backend_testing::init_no_event_loop();
    let ops: Vec<PatchOp> = serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("patches/exp03-sidebar-280.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let spec = apply_patch(&file_manager_spec(), &ops).unwrap();
    let (session, _app) = create(&spec).expect("window created");
    session.instance().show().expect("shows");
    let diags = check_geometry(session.instance(), &spec);
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
}

#[test]
fn child_wider_than_its_sidebar_is_reported_as_overflow_from_real_layout() {
    i_slint_backend_testing::init_no_event_loop();
    let mut spec: UiSpec = file_manager_spec();
    // Deliberate defect: a 400 dp label inside the 240 dp sidebar.
    let sidebar = find_mut(&mut spec.root, "sidebar").unwrap();
    sidebar.children.push(
        Node::new("too_wide_label", Kind::Leaf)
            .with_width(Length::Dp(400.0))
            .with_widget(Widget::Text {
                content: "ui.all_files".to_string(),
            }),
    );
    let (session, _app) = create(&spec).expect("window created");
    session.instance().show().expect("shows");
    let diags = check_geometry(session.instance(), &spec);
    let overflow: Vec<&GeometryDiagnostic> = diags
        .iter()
        .filter(|d| d.code == "LAYOUT_OVERFLOW")
        .collect();
    assert!(
        !overflow.is_empty(),
        "expected LAYOUT_OVERFLOW, got {:?}",
        codes(&diags)
    );
    let d = overflow
        .iter()
        .find(|d| d.node == "too_wide_label")
        .expect("node reported");
    assert!(d.overflow > 0.0, "{d:?}");
}
