use aster_backend_text::render;
use aster_ui_model::{apply_patch, Kind, Length, Node, PatchOp, UiSpec};

fn tiny_spec() -> UiSpec {
    UiSpec {
        title: "T".to_string(),
        min_height: 500.0,
        preferred_width: 1100.0,
        preferred_height: 720.0,
        root: Node::new("main", Kind::Window).with_children(vec![Node::new(
            "sidebar",
            Kind::Column,
        )
        .with_width(Length::Dp(240.0))]),
        state: vec![],
        callbacks: vec![],
    }
}

#[test]
fn text_backend_lists_nodes_with_model_names() {
    let text = render(&tiny_spec());
    assert!(text.contains("sidebar kind=Column width=240dp"), "{text}");
    assert!(text.starts_with("window title=\"T\""));
}

#[test]
fn text_backend_reflects_a_structured_patch_on_the_same_model() {
    let spec = tiny_spec();
    let ops = vec![PatchOp::SetProp {
        target: "sidebar".to_string(),
        name: "width".to_string(),
        value: "280dp".to_string(),
        expect: Some("240dp".to_string()),
    }];
    let patched = apply_patch(&spec, &ops).unwrap();
    let before = render(&spec);
    let after = render(&patched);
    let changed = before
        .lines()
        .zip(after.lines())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(changed, 1);
    assert!(after.contains("sidebar kind=Column width=280dp"));
}
