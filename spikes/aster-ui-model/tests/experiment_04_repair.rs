use aster_ui_model::{check_window, find_mut, required_min_width, Kind, Length, Node};

fn broken() -> Node {
    // Deliberately broken: content minimum is too large for an 800 dp window.
    Node::new("main", Kind::Row)
        .with_padding(48.0)
        .with_children(vec![
            Node::new("sidebar", Kind::Leaf)
                .with_width(Length::Dp(240.0))
                .with_min_width(240.0),
            Node::new("content", Kind::Leaf)
                .with_width(Length::Fill)
                .with_min_width(560.0),
        ])
}

#[test]
fn broken_layout_reports_structured_conflict() {
    let d = check_window(&broken(), 800.0);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].code, "LAYOUT_CONFLICT");
    assert!(d[0]
        .suggestions
        .iter()
        .any(|s| s.contains("content.min_width")));
}

#[test]
fn suggested_patch_resolves_conflict_without_changing_other_nodes() {
    // Patch: lower content.min_width from 560 to 400 (the suggested edit).
    let mut root = broken();
    let before_sidebar = find_mut(&mut root, "sidebar").unwrap().clone();
    find_mut(&mut root, "content").unwrap().min_width = 400.0;

    assert!(check_window(&root, 800.0).is_empty());
    assert_eq!(*find_mut(&mut root, "sidebar").unwrap(), before_sidebar);
    assert_eq!(required_min_width(&root), 736.0);
}

#[test]
fn patch_that_does_not_fix_root_cause_is_rejected_by_check() {
    // Lowering only the sidebar does not fix the conflict if content is still too large.
    let mut root = broken();
    find_mut(&mut root, "sidebar").unwrap().min_width = 200.0;
    assert!(!check_window(&root, 800.0).is_empty());
}
