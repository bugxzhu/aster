//! Slint adapter for the Aster UI model (UI Spike 01, prototype B).
//!
//! Dependency direction: this crate depends on `aster-ui-model`. The model never depends on
//! this crate or on Slint (06 section 1.2, 02 section 11.5). The adapter turns a `UiSpec` into
//! `.slint` source and runs it with the Slint interpreter, so a changed model takes effect
//! without recompiling Rust.

#[cfg(feature = "layout-probe")]
pub mod probe;

use aster_ui_model::{FieldType, Kind, Length, Node, UiSpec, Widget};
use std::fmt::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum CodegenError {
    /// The node combines a kind and a widget that the adapter cannot express.
    UnsupportedNode(String),
}

/// Maps a model key such as `ui.all_files` to a Slint property name `ui-all-files`.
pub fn prop_name(key: &str) -> String {
    key.replace(['.', '_'], "-")
}

pub fn element_id(id: &str) -> String {
    id.replace('_', "-")
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Generates Slint source for the spec. Output is deterministic for a given spec.
pub fn to_slint_source(spec: &UiSpec) -> Result<String, CodegenError> {
    let mut out = String::new();
    out.push_str("import { Button, LineEdit, ListView } from \"std-widgets.slint\";\n\n");
    if spec.state.iter().any(|f| f.ty == FieldType::FileRows) {
        out.push_str("export struct FileRow {\n    name: string,\n    kind: string,\n    modified: string,\n}\n\n");
    }
    out.push_str("export component MainWindow inherits Window {\n");
    let _ = writeln!(out, "    title: \"{}\";", escape(&spec.title));
    let _ = writeln!(out, "    min-width: {}px;", spec.root.min_width);
    let _ = writeln!(out, "    min-height: {}px;", spec.min_height);
    let _ = writeln!(out, "    preferred-width: {}px;", spec.preferred_width);
    let _ = writeln!(out, "    preferred-height: {}px;", spec.preferred_height);
    for f in &spec.state {
        let ty = match f.ty {
            FieldType::Text => "string",
            FieldType::FileRows => "[FileRow]",
        };
        let _ = writeln!(out, "    in-out property <{ty}> {};", prop_name(&f.name));
    }
    for c in &spec.callbacks {
        let arg = if c.takes_text { "string" } else { "" };
        let _ = writeln!(out, "    callback {}({arg});", prop_name(&c.name));
    }
    for child in &spec.root.children {
        emit_node(&mut out, child, 1)?;
    }
    out.push_str("}\n");
    Ok(out)
}

fn layout_props(node: &Node, indent: &str, out: &mut String) {
    match node.width {
        Length::Dp(v) => {
            let _ = writeln!(out, "{indent}    width: {v}px;");
        }
        Length::Fill => {
            let _ = writeln!(out, "{indent}    horizontal-stretch: 1;");
        }
        Length::Wrap => {}
    }
    if node.min_width > 0.0 {
        let _ = writeln!(out, "{indent}    min-width: {}px;", node.min_width);
    }
    if node.padding > 0.0 {
        let _ = writeln!(out, "{indent}    padding: {}px;", node.padding);
    }
    if node.gap > 0.0 {
        let _ = writeln!(out, "{indent}    spacing: {}px;", node.gap);
    }
}

fn emit_node(out: &mut String, node: &Node, depth: usize) -> Result<(), CodegenError> {
    let indent = "    ".repeat(depth);
    let id = element_id(&node.id);
    match (&node.kind, &node.widget) {
        (Kind::Row, None) | (Kind::Column, None) => {
            let element = if node.kind == Kind::Row {
                "HorizontalLayout"
            } else {
                "VerticalLayout"
            };
            let _ = writeln!(out, "{indent}{id} := {element} {{");
            layout_props(node, &indent, out);
            for child in &node.children {
                emit_node(out, child, depth + 1)?;
            }
            let _ = writeln!(out, "{indent}}}");
        }
        (Kind::Leaf, Some(widget)) => {
            let _ = writeln!(out, "{indent}{id} := {} {{", element_name(widget));
            layout_props(node, &indent, out);
            emit_widget(out, widget, &indent);
            let _ = writeln!(out, "{indent}}}");
        }
        _ => return Err(CodegenError::UnsupportedNode(node.id.clone())),
    }
    Ok(())
}

fn element_name(w: &Widget) -> &'static str {
    match w {
        Widget::Text { .. } => "Text",
        Widget::Button { .. } => "Button",
        Widget::LineEdit { .. } => "LineEdit",
        Widget::FileList { .. } => "ListView",
    }
}

fn emit_widget(out: &mut String, w: &Widget, indent: &str) {
    let i = format!("{indent}    ");
    match w {
        Widget::Text { content } => {
            let _ = writeln!(out, "{i}text: root.{};", prop_name(content));
        }
        Widget::Button { label, on_click } => {
            let _ = writeln!(out, "{i}text: root.{};", prop_name(label));
            let _ = writeln!(out, "{i}clicked => {{ root.{}(); }}", prop_name(on_click));
        }
        Widget::LineEdit {
            placeholder,
            value,
            on_edit,
        } => {
            let _ = writeln!(out, "{i}placeholder-text: root.{};", prop_name(placeholder));
            let _ = writeln!(out, "{i}text <=> root.{};", prop_name(value));
            let _ = writeln!(
                out,
                "{i}edited => {{ root.{}(self.text); }}",
                prop_name(on_edit)
            );
        }
        Widget::FileList { items, on_select } => {
            let _ = writeln!(
                out,
                "{i}for file in root.{} : Rectangle {{",
                prop_name(items)
            );
            let _ = writeln!(out, "{i}    height: 32px;");
            let _ = writeln!(
                out,
                "{i}    TouchArea {{ clicked => {{ root.{}(file.name); }} }}",
                prop_name(on_select)
            );
            let _ = writeln!(out, "{i}    HorizontalLayout {{");
            let _ = writeln!(out, "{i}        padding-left: 8px;");
            let _ = writeln!(
                out,
                "{i}        Text {{ text: file.name; overflow: elide; horizontal-stretch: 1; }}"
            );
            let _ = writeln!(out, "{i}        Text {{ text: file.kind; width: 120px; }}");
            let _ = writeln!(
                out,
                "{i}        Text {{ text: file.modified; width: 100px; }}"
            );
            let _ = writeln!(out, "{i}    }}");
            let _ = writeln!(out, "{i}}}");
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    Codegen(CodegenError),
    Invalid(aster_ui_model::SpecError),
    Compile(Vec<String>),
    Instantiate(String),
}

/// A running instance of a spec. Owns the Slint component; the business state stays outside.
pub struct Session {
    instance: slint_interpreter::ComponentInstance,
}

/// Validates the spec, generates source, compiles it and creates the window (not yet shown).
pub fn load(spec: &UiSpec) -> Result<Session, LoadError> {
    spec.validate().map_err(LoadError::Invalid)?;
    let source = to_slint_source(spec).map_err(LoadError::Codegen)?;
    let compiler = slint_interpreter::Compiler::new();
    let result = pollster::block_on(
        compiler.build_from_source(source, PathBuf::from("aster-generated.slint")),
    );
    if result.has_errors() {
        let messages = result.diagnostics().map(|d| d.to_string()).collect();
        return Err(LoadError::Compile(messages));
    }
    let definition = result
        .component("MainWindow")
        .ok_or_else(|| LoadError::Compile(vec!["component MainWindow not found".to_string()]))?;
    let instance = definition
        .create()
        .map_err(|e| LoadError::Instantiate(e.to_string()))?;
    Ok(Session { instance })
}

impl Session {
    pub fn instance(&self) -> &slint_interpreter::ComponentInstance {
        &self.instance
    }

    pub fn set_text(&self, key: &str, value: &str) -> Result<(), String> {
        self.instance
            .set_property(
                &prop_name(key),
                slint_interpreter::Value::String(value.into()),
            )
            .map_err(|e| format!("{key}: {e}"))
    }

    /// Sets a file-row list. Each row is `[name, kind, modified]`.
    pub fn set_file_rows(&self, key: &str, rows: &[[String; 3]]) -> Result<(), String> {
        let values: Vec<slint_interpreter::Value> = rows
            .iter()
            .map(|r| {
                let mut s = slint_interpreter::Struct::default();
                s.set_field(
                    "name".to_string(),
                    slint_interpreter::Value::String(r[0].as_str().into()),
                );
                s.set_field(
                    "kind".to_string(),
                    slint_interpreter::Value::String(r[1].as_str().into()),
                );
                s.set_field(
                    "modified".to_string(),
                    slint_interpreter::Value::String(r[2].as_str().into()),
                );
                slint_interpreter::Value::Struct(s)
            })
            .collect();
        let model = slint::ModelRc::new(slint::VecModel::from(values));
        self.instance
            .set_property(&prop_name(key), slint_interpreter::Value::Model(model))
            .map_err(|e| format!("{key}: {e}"))
    }

    pub fn on_text_callback(&self, name: &str, f: impl Fn(&str) + 'static) -> Result<(), String> {
        self.instance
            .set_callback(&prop_name(name), move |args| {
                if let Some(slint_interpreter::Value::String(s)) = args.first() {
                    f(s.as_str());
                }
                slint_interpreter::Value::Void
            })
            .map_err(|e| format!("{name}: {e}"))
    }

    pub fn on_unit_callback(&self, name: &str, f: impl Fn() + 'static) -> Result<(), String> {
        self.instance
            .set_callback(&prop_name(name), move |_| {
                f();
                slint_interpreter::Value::Void
            })
            .map_err(|e| format!("{name}: {e}"))
    }
}
