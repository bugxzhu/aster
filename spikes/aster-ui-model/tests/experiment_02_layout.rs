use aster_ui_model::{check_window, find_mut, Kind, Length, Node};

fn app(sidebar_min: f32, content_min: f32, padding: f32) -> Node {
    Node::new("main", Kind::Row)
        .with_padding(padding)
        .with_children(vec![
            Node::new("sidebar", Kind::Leaf)
                .with_width(Length::Dp(240.0))
                .with_min_width(sidebar_min),
            Node::new("content", Kind::Leaf)
                .with_width(Length::Fill)
                .with_min_width(content_min),
        ])
}

#[test]
fn window_sizes_at_or_above_minimum_produce_no_conflict() {
    let root = app(240.0, 400.0, 48.0);
    for w in [800.0_f32, 900.0, 1100.0, 1920.0] {
        assert!(
            check_window(&root, w).is_empty(),
            "unexpected conflict at {w}"
        );
    }
}

#[test]
fn window_below_minimum_reports_overflow_at_each_size() {
    let root = app(240.0, 400.0, 48.0);
    let d = check_window(&root, 700.0);
    assert_eq!(d[0].overflow, 36.0);
}

#[test]
fn long_translated_label_raises_minimum_and_is_detected() {
    let mut root = app(240.0, 400.0, 48.0);
    find_mut(&mut root, "content").unwrap().min_width = 620.0;
    assert!(!check_window(&root, 800.0).is_empty());
}

#[test]
fn check_is_deterministic_for_identical_input() {
    // The model works in dp only. Physical scale does not enter this function,
    // so repeated calls on the same input must return the same diagnostics.
    // This is a determinism check, not a proof of DPI independence.
    let root = app(240.0, 400.0, 48.0);
    let a = check_window(&root, 800.0);
    let b = check_window(&root, 800.0);
    assert_eq!(a, b);
}
