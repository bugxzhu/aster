//! Second backend for the Aster UI model (experiment 05). It renders the same `UiSpec` as a
//! text tree instead of a window. It has no dependency on Slint, which shows that the model
//! and the business layer do not depend on the Slint adapter.

use aster_ui_model::{Kind, Length, Node, UiSpec, Widget};
use std::fmt::Write;

fn length_text(l: Length) -> String {
    match l {
        Length::Dp(v) => format!("{v}dp"),
        Length::Fill => "fill".to_string(),
        Length::Wrap => "wrap".to_string(),
    }
}

fn kind_text(k: Kind) -> &'static str {
    match k {
        Kind::Window => "Window",
        Kind::Row => "Row",
        Kind::Column => "Column",
        Kind::Leaf => "Leaf",
    }
}

fn widget_text(w: &Widget) -> String {
    match w {
        Widget::Text { content } => format!("text({content})"),
        Widget::Button { label, on_click } => format!("button(label={label}, on_click={on_click})"),
        Widget::LineEdit {
            placeholder,
            value,
            on_edit,
        } => {
            format!("line_edit(placeholder={placeholder}, value={value}, on_edit={on_edit})")
        }
        Widget::FileList { items, on_select } => {
            format!("file_list(items={items}, on_select={on_select})")
        }
    }
}

fn render_node(out: &mut String, node: &Node, depth: usize) {
    let indent = "  ".repeat(depth);
    let _ = write!(
        out,
        "{indent}{} kind={} width={} min_width={}dp padding={}dp gap={}dp",
        node.id,
        kind_text(node.kind),
        length_text(node.width),
        node.min_width,
        node.padding,
        node.gap
    );
    if let Some(w) = &node.widget {
        let _ = write!(out, " {}", widget_text(w));
    }
    out.push('\n');
    for c in &node.children {
        render_node(out, c, depth + 1);
    }
}

/// Renders the spec as an indented text tree with the same node and binding names as the model.
pub fn render(spec: &UiSpec) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "window title={:?} min_height={}dp preferred={}x{}dp",
        spec.title, spec.min_height, spec.preferred_width, spec.preferred_height
    );
    render_node(&mut out, &spec.root, 0);
    out
}
