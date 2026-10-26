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

// The following checks were added after a screenshot showed that the generated layout stretched
// toolbar buttons, the search field and sidebar labels over the full window height. Widths alone
// did not catch it, so these checks assert heights and vertical order in the real layout.

fn rect(session: &aster_backend_slint::Session, id: &str) -> aster_backend_slint::probe::Rect {
    aster_backend_slint::probe::rect_of(session.instance(), id)
        .unwrap_or_else(|| panic!("no element {id}"))
}

#[test]
fn toolbar_keeps_its_intrinsic_height_in_the_real_layout() {
    i_slint_backend_testing::init_no_event_loop();
    let (session, _app) = create(&file_manager_spec()).expect("window created");
    session.instance().show().expect("shows");
    let toolbar = rect(&session, "toolbar");
    assert!(
        toolbar.height < 60.0,
        "toolbar stretched to {} dp",
        toolbar.height
    );
}

#[test]
fn sidebar_labels_stack_from_the_top_of_the_sidebar() {
    i_slint_backend_testing::init_no_event_loop();
    let (session, _app) = create(&file_manager_spec()).expect("window created");
    session.instance().show().expect("shows");
    let first = rect(&session, "all_files_label");
    let second = rect(&session, "recent_label");
    assert!(first.y < 40.0, "first label not at top: y={}", first.y);
    assert!(
        second.y - first.y < 40.0,
        "labels spread apart: {} vs {}",
        first.y,
        second.y
    );
}

#[test]
fn search_field_has_a_single_line_height() {
    i_slint_backend_testing::init_no_event_loop();
    let (session, _app) = create(&file_manager_spec()).expect("window created");
    session.instance().show().expect("shows");
    let search = rect(&session, "search");
    assert!(
        search.height < 60.0,
        "search field stretched to {} dp",
        search.height
    );
}

#[test]
fn file_list_fills_the_remaining_height_of_the_content_area() {
    i_slint_backend_testing::init_no_event_loop();
    let (session, _app) = create(&file_manager_spec()).expect("window created");
    session.instance().show().expect("shows");
    let list = rect(&session, "file_list");
    let window_height = session.instance().window().size().height as f32
        / session.instance().window().scale_factor();
    // Window minus toolbar, search field and paddings leaves well over 300 dp.
    assert!(
        list.height > 300.0,
        "file list only {} dp of {} dp window",
        list.height,
        window_height
    );
}

#[test]
fn minimum_window_800_by_500_has_no_layout_diagnostics() {
    // Requirement F1: minimum 800 x 500 logical px. Checked on the real layout at that size.
    i_slint_backend_testing::init_no_event_loop();
    let spec = file_manager_spec();
    let (session, _app) = create(&spec).expect("window created");
    session.instance().show().expect("shows");
    session
        .instance()
        .window()
        .set_size(slint::LogicalSize::new(800.0, 500.0));
    let size = session.instance().window().size();
    let scale = session.instance().window().scale_factor();
    assert_eq!(
        (size.width as f32 / scale, size.height as f32 / scale),
        (800.0, 500.0),
        "window was not resized"
    );
    let diags = aster_backend_slint::probe::check_geometry(session.instance(), &spec);
    assert!(diags.is_empty(), "diagnostics at 800x500: {diags:?}");
    let list = rect(&session, "file_list");
    assert!(
        list.height > 200.0,
        "list too short at minimum size: {}",
        list.height
    );
}
