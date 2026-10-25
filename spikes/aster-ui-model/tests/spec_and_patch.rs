use aster_ui_model::{
    apply_patch, property_text, CallbackSpec, FieldType, Kind, Length, Node, PatchError, PatchOp,
    SpecError, StateField, UiSpec, Widget,
};

fn sample_spec() -> UiSpec {
    let sidebar = Node::new("sidebar", Kind::Column)
        .with_width(Length::Dp(240.0))
        .with_padding(16.0)
        .with_children(vec![Node::new("all_files_label", Kind::Leaf).with_widget(Widget::Text {
            content: "ui.all_files".to_string(),
        })]);
    let content = Node::new("content", Kind::Column)
        .with_width(Length::Fill)
        .with_children(vec![
            Node::new("search", Kind::Leaf).with_widget(Widget::LineEdit {
                placeholder: "ui.search".to_string(),
                value: "query".to_string(),
                on_edit: "search_changed".to_string(),
            }),
            Node::new("list", Kind::Leaf).with_widget(Widget::FileList {
                items: "files".to_string(),
                on_select: "file_selected".to_string(),
            }),
        ]);
    let root = Node::new("main", Kind::Window)
        .with_min_width(800.0)
        .with_children(vec![Node::new("body", Kind::Row).with_children(vec![sidebar, content])]);
    UiSpec {
        title: "File Manager".to_string(),
        min_height: 500.0,
        preferred_width: 1100.0,
        preferred_height: 720.0,
        root,
        state: vec![
            StateField { name: "ui.all_files".to_string(), ty: FieldType::Text },
            StateField { name: "ui.search".to_string(), ty: FieldType::Text },
            StateField { name: "query".to_string(), ty: FieldType::Text },
            StateField { name: "files".to_string(), ty: FieldType::FileRows },
        ],
        callbacks: vec![
            CallbackSpec { name: "search_changed".to_string(), takes_text: true },
            CallbackSpec { name: "file_selected".to_string(), takes_text: true },
        ],
    }
}

#[test]
fn consistent_spec_validates() {
    assert_eq!(sample_spec().validate(), Ok(()));
}

#[test]
fn unknown_state_key_is_reported_with_node_id() {
    let mut spec = sample_spec();
    spec.state.retain(|f| f.name != "ui.search");
    assert_eq!(
        spec.validate(),
        Err(SpecError::UnknownStateKey { node: "search".to_string(), key: "ui.search".to_string() })
    );
}

#[test]
fn binding_to_wrong_field_type_is_rejected() {
    // Binding a FileList to a text field must fail, as required by 02 section 7.2.
    let mut spec = sample_spec();
    spec.state.iter_mut().find(|f| f.name == "files").unwrap().ty = FieldType::Text;
    assert_eq!(
        spec.validate(),
        Err(SpecError::BindingTypeMismatch { node: "list".to_string(), key: "files".to_string() })
    );
}

#[test]
fn callback_with_wrong_argument_shape_is_rejected() {
    let mut spec = sample_spec();
    spec.callbacks.iter_mut().find(|c| c.name == "file_selected").unwrap().takes_text = false;
    assert_eq!(
        spec.validate(),
        Err(SpecError::CallbackArgumentMismatch {
            node: "list".to_string(),
            name: "file_selected".to_string()
        })
    );
}

#[test]
fn duplicate_ids_are_rejected() {
    let mut spec = sample_spec();
    spec.root.children[0].children[0].id = "content".to_string();
    assert_eq!(spec.validate(), Err(SpecError::DuplicateId("content".to_string())));
}

#[test]
fn patch_changes_sidebar_width_and_keeps_original_unchanged() {
    let spec = sample_spec();
    let ops: Vec<PatchOp> = serde_json::from_str(
        r#"[{"op":"set_prop","target":"sidebar","name":"width","value":"280dp","expect":"240dp"}]"#,
    )
    .unwrap();
    let next = apply_patch(&spec, &ops).unwrap();
    assert_eq!(property_text(&next, "sidebar", "width").as_deref(), Some("280dp"));
    assert_eq!(property_text(&spec, "sidebar", "width").as_deref(), Some("240dp"));
    assert_eq!(next.validate(), Ok(()));
}

#[test]
fn stale_expect_is_rejected_as_conflict() {
    let spec = sample_spec();
    let ops = vec![PatchOp::SetProp {
        target: "sidebar".to_string(),
        name: "width".to_string(),
        value: "280dp".to_string(),
        expect: Some("260dp".to_string()),
    }];
    assert_eq!(
        apply_patch(&spec, &ops),
        Err(PatchError::Conflict {
            target: "sidebar".to_string(),
            name: "width".to_string(),
            expected: "260dp".to_string(),
            actual: "240dp".to_string(),
        })
    );
}

#[test]
fn unknown_target_and_property_are_errors() {
    let spec = sample_spec();
    let missing = vec![PatchOp::SetProp {
        target: "nope".to_string(),
        name: "width".to_string(),
        value: "1dp".to_string(),
        expect: None,
    }];
    assert_eq!(apply_patch(&spec, &missing), Err(PatchError::TargetNotFound("nope".to_string())));

    let bad_prop = vec![PatchOp::SetProp {
        target: "sidebar".to_string(),
        name: "color".to_string(),
        value: "red".to_string(),
        expect: None,
    }];
    assert_eq!(
        apply_patch(&spec, &bad_prop),
        Err(PatchError::UnknownProperty("color".to_string()))
    );
}

#[test]
fn failed_multi_op_patch_applies_nothing() {
    // First op is valid, second has a stale expect: the whole patch must be rejected.
    let spec = sample_spec();
    let ops = vec![
        PatchOp::SetProp {
            target: "sidebar".to_string(),
            name: "width".to_string(),
            value: "280dp".to_string(),
            expect: None,
        },
        PatchOp::SetProp {
            target: "content".to_string(),
            name: "gap".to_string(),
            value: "8dp".to_string(),
            expect: Some("99dp".to_string()),
        },
    ];
    assert!(apply_patch(&spec, &ops).is_err());
    assert_eq!(property_text(&spec, "sidebar", "width").as_deref(), Some("240dp"));
}
