//! The File Manager window described as an Aster UI model. This is the only place where the
//! layout and bindings of the window are written down; the Slint source is generated from it.

use aster_ui_model::{CallbackSpec, FieldType, Kind, Length, Node, StateField, UiSpec, Widget};

fn text_label(id: &str, key: &str) -> Node {
    Node::new(id, Kind::Leaf).with_widget(Widget::Text { content: key.to_string() })
}

fn button(id: &str, label_key: &str, on_click: &str) -> Node {
    Node::new(id, Kind::Leaf).with_widget(Widget::Button {
        label: label_key.to_string(),
        on_click: on_click.to_string(),
    })
}

pub fn file_manager_spec() -> UiSpec {
    let sidebar = Node::new("sidebar", Kind::Column)
        .with_width(Length::Dp(240.0))
        .with_padding(16.0)
        .with_gap(8.0)
        .with_children(vec![
            text_label("all_files_label", "ui.all_files"),
            text_label("recent_label", "ui.recent"),
            text_label("favorites_label", "ui.favorites"),
        ]);

    let toolbar = Node::new("toolbar", Kind::Row).with_gap(8.0).with_children(vec![
        button("new_folder_button", "ui.new_folder", "new_folder_clicked"),
        button("refresh_button", "ui.refresh", "refresh_clicked"),
        button("theme_button", "ui.theme_label", "theme_toggled"),
        button("language_button", "ui.language", "language_toggled"),
    ]);

    let search = Node::new("search", Kind::Leaf).with_widget(Widget::LineEdit {
        placeholder: "ui.search_placeholder".to_string(),
        value: "query".to_string(),
        on_edit: "search_changed".to_string(),
    });

    let list = Node::new("file_list", Kind::Leaf)
        .with_width(Length::Fill)
        .with_widget(Widget::FileList {
            items: "files".to_string(),
            on_select: "file_selected".to_string(),
        });

    let detail = Node::new("detail", Kind::Column)
        .with_width(Length::Dp(280.0))
        .with_padding(12.0)
        .with_children(vec![Node::new("detail_text", Kind::Leaf)
            .with_widget(Widget::Text { content: "detail_text".to_string() })]);

    let main_row = Node::new("main_row", Kind::Row)
        .with_gap(12.0)
        .with_children(vec![list, detail]);

    let content = Node::new("content", Kind::Column)
        .with_width(Length::Fill)
        .with_padding(16.0)
        .with_gap(12.0)
        .with_children(vec![toolbar, search, main_row]);

    let root = Node::new("main", Kind::Window)
        .with_min_width(800.0)
        .with_children(vec![Node::new("body", Kind::Row).with_children(vec![sidebar, content])]);

    let text = |name: &str| StateField { name: name.to_string(), ty: FieldType::Text };
    UiSpec {
        title: "File Manager".to_string(),
        min_height: 500.0,
        preferred_width: 1100.0,
        preferred_height: 720.0,
        root,
        state: vec![
            text("ui.all_files"),
            text("ui.recent"),
            text("ui.favorites"),
            text("ui.new_folder"),
            text("ui.refresh"),
            text("ui.theme_label"),
            text("ui.language"),
            text("ui.search_placeholder"),
            text("query"),
            text("detail_text"),
            StateField { name: "files".to_string(), ty: FieldType::FileRows },
        ],
        callbacks: vec![
            CallbackSpec { name: "new_folder_clicked".to_string(), takes_text: false },
            CallbackSpec { name: "refresh_clicked".to_string(), takes_text: false },
            CallbackSpec { name: "theme_toggled".to_string(), takes_text: false },
            CallbackSpec { name: "language_toggled".to_string(), takes_text: false },
            CallbackSpec { name: "search_changed".to_string(), takes_text: true },
            CallbackSpec { name: "file_selected".to_string(), takes_text: true },
        ],
    }
}
