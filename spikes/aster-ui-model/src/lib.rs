//! Minimal Aster UI model for UI Spike 01 (prototype B).
//! This crate must not depend on any rendering backend (06 section 1.2).

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Dp(f32),
    Fill,
    Wrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Window,
    Row,
    Column,
    Leaf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: String,
    pub kind: Kind,
    pub width: Length,
    pub min_width: f32,
    /// Padding applied on both horizontal sides, in dp.
    pub padding: f32,
    pub gap: f32,
    pub children: Vec<Node>,
    /// Leaf content. `None` for containers.
    pub widget: Option<Widget>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: &'static str,
    pub component: String,
    pub property: &'static str,
    pub actual: f32,
    pub expected: f32,
    pub overflow: f32,
    pub suggestions: Vec<String>,
}

impl Node {
    pub fn new(id: &str, kind: Kind) -> Self {
        Node {
            id: id.to_string(),
            kind,
            width: Length::Wrap,
            min_width: 0.0,
            padding: 0.0,
            gap: 0.0,
            children: Vec::new(),
            widget: None,
        }
    }
    pub fn with_width(mut self, w: Length) -> Self {
        self.width = w;
        self
    }
    pub fn with_min_width(mut self, w: f32) -> Self {
        self.min_width = w;
        self
    }
    pub fn with_padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }
    pub fn with_gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }
    pub fn with_widget(mut self, w: Widget) -> Self {
        self.widget = Some(w);
        self
    }
    pub fn with_children(mut self, c: Vec<Node>) -> Self {
        self.children = c;
        self
    }
}

impl Diagnostic {
    pub fn to_json(&self) -> String {
        // INVARIANT: Diagnostic contains only plain data and always serializes.
        serde_json::to_string(self).expect("INVARIANT: Diagnostic is always serializable")
    }
}

/// Minimum width the node needs, in dp.
pub fn required_min_width(node: &Node) -> f32 {
    let own = match node.kind {
        Kind::Row => {
            let n = node.children.len();
            let sum: f32 = node.children.iter().map(required_min_width).sum();
            let gaps = if n > 1 {
                node.gap * (n as f32 - 1.0)
            } else {
                0.0
            };
            sum + gaps
        }
        Kind::Column | Kind::Window => node
            .children
            .iter()
            .map(required_min_width)
            .fold(0.0, f32::max),
        Kind::Leaf => 0.0,
    };
    node.min_width.max(own + 2.0 * node.padding)
}

/// Checks the root against the window width and reports a LAYOUT_CONFLICT when it does not fit.
pub fn check_window(root: &Node, window_width: f32) -> Vec<Diagnostic> {
    let required = required_min_width(root);
    if required <= window_width {
        return Vec::new();
    }
    vec![Diagnostic {
        code: "LAYOUT_CONFLICT",
        severity: "error",
        component: root.id.clone(),
        property: "min_width",
        actual: required,
        expected: window_width,
        overflow: required - window_width,
        suggestions: vec![
            "reduce padding on root".to_string(),
            "lower content.min_width".to_string(),
            "allow sidebar to collapse".to_string(),
        ],
    }]
}

/// Depth-first search for the first node with the given id.
pub fn find_mut<'a>(root: &'a mut Node, id: &str) -> Option<&'a mut Node> {
    if root.id == id {
        return Some(root);
    }
    for child in root.children.iter_mut() {
        if let Some(found) = find_mut(child, id) {
            return Some(found);
        }
    }
    None
}

/// Leaf controls. Bindings are state keys (for example `ui.all_files`); callbacks are names.
/// The model never refers to a rendering backend (02 section 7.2, 06 section 1.2).
#[derive(Debug, Clone, PartialEq)]
pub enum Widget {
    /// Read-only text bound to a text state field.
    Text { content: String },
    /// Button whose label is a text state field; `on_click` takes no argument.
    Button { label: String, on_click: String },
    /// Single-line editor. `value` is two-way bound; `on_edit` receives the new text.
    LineEdit {
        placeholder: String,
        value: String,
        on_edit: String,
    },
    /// Vertical list of file rows bound to a row-list state field; `on_select` receives the file name.
    FileList { items: String, on_select: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Text,
    FileRows,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateField {
    pub name: String,
    pub ty: FieldType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallbackSpec {
    pub name: String,
    /// True if the callback receives one text argument.
    pub takes_text: bool,
}

/// A complete window description: structure, state declarations and callbacks.
#[derive(Debug, Clone, PartialEq)]
pub struct UiSpec {
    pub title: String,
    pub min_height: f32,
    pub preferred_width: f32,
    pub preferred_height: f32,
    /// Root node has `Kind::Window`; `min_width` is the window minimum width.
    pub root: Node,
    pub state: Vec<StateField>,
    pub callbacks: Vec<CallbackSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecError {
    DuplicateId(String),
    UnknownStateKey { node: String, key: String },
    BindingTypeMismatch { node: String, key: String },
    UnknownCallback { node: String, name: String },
    CallbackArgumentMismatch { node: String, name: String },
}

impl UiSpec {
    /// Checks ids, state bindings (including field types) and callback signatures.
    pub fn validate(&self) -> Result<(), SpecError> {
        let mut ids = std::collections::HashSet::new();
        self.validate_node(&self.root, &mut ids)
    }

    fn validate_node(
        &self,
        node: &Node,
        ids: &mut std::collections::HashSet<String>,
    ) -> Result<(), SpecError> {
        if !ids.insert(node.id.clone()) {
            return Err(SpecError::DuplicateId(node.id.clone()));
        }
        if let Some(w) = &node.widget {
            self.validate_widget(&node.id, w)?;
        }
        for c in &node.children {
            self.validate_node(c, ids)?;
        }
        Ok(())
    }

    fn field(&self, node: &str, key: &str, expected: FieldType) -> Result<(), SpecError> {
        match self.state.iter().find(|f| f.name == key) {
            None => Err(SpecError::UnknownStateKey {
                node: node.to_string(),
                key: key.to_string(),
            }),
            Some(f) if f.ty != expected => Err(SpecError::BindingTypeMismatch {
                node: node.to_string(),
                key: key.to_string(),
            }),
            Some(_) => Ok(()),
        }
    }

    fn callback(&self, node: &str, name: &str, takes_text: bool) -> Result<(), SpecError> {
        match self.callbacks.iter().find(|c| c.name == name) {
            None => Err(SpecError::UnknownCallback {
                node: node.to_string(),
                name: name.to_string(),
            }),
            Some(c) if c.takes_text != takes_text => Err(SpecError::CallbackArgumentMismatch {
                node: node.to_string(),
                name: name.to_string(),
            }),
            Some(_) => Ok(()),
        }
    }

    fn validate_widget(&self, node: &str, w: &Widget) -> Result<(), SpecError> {
        match w {
            Widget::Text { content } => self.field(node, content, FieldType::Text),
            Widget::Button { label, on_click } => {
                self.field(node, label, FieldType::Text)?;
                self.callback(node, on_click, false)
            }
            Widget::LineEdit {
                placeholder,
                value,
                on_edit,
            } => {
                self.field(node, placeholder, FieldType::Text)?;
                self.field(node, value, FieldType::Text)?;
                self.callback(node, on_edit, true)
            }
            Widget::FileList { items, on_select } => {
                self.field(node, items, FieldType::FileRows)?;
                self.callback(node, on_select, true)
            }
        }
    }
}

/// Structured patch operation (03 section 6). Serialized as JSON with an `op` tag.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum PatchOp {
    /// Set one property. `value` and `expect` use the textual forms "240dp", "fill", "wrap".
    SetProp {
        target: String,
        name: String,
        value: String,
        #[serde(default)]
        expect: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatchError {
    TargetNotFound(String),
    UnknownProperty(String),
    BadValue(String),
    /// The current value differs from `expect`; the patch is rejected (03 section 6.1).
    Conflict {
        target: String,
        name: String,
        expected: String,
        actual: String,
    },
}

fn parse_dp(s: &str) -> Option<f32> {
    s.strip_suffix("dp")?.parse::<f32>().ok()
}

fn parse_length(s: &str) -> Option<Length> {
    match s {
        "fill" => Some(Length::Fill),
        "wrap" => Some(Length::Wrap),
        _ => parse_dp(s).map(Length::Dp),
    }
}

fn read_prop(node: &Node, name: &str) -> Result<String, PatchError> {
    match name {
        "width" => Ok(match node.width {
            Length::Dp(v) => format!("{v}dp"),
            Length::Fill => "fill".to_string(),
            Length::Wrap => "wrap".to_string(),
        }),
        "min_width" => Ok(format!("{}dp", node.min_width)),
        "padding" => Ok(format!("{}dp", node.padding)),
        "gap" => Ok(format!("{}dp", node.gap)),
        other => Err(PatchError::UnknownProperty(other.to_string())),
    }
}

fn write_prop(node: &mut Node, name: &str, value: &str) -> Result<(), PatchError> {
    let bad = || PatchError::BadValue(format!("{name}={value}"));
    match name {
        "width" => node.width = parse_length(value).ok_or_else(bad)?,
        "min_width" => node.min_width = parse_dp(value).ok_or_else(bad)?,
        "padding" => node.padding = parse_dp(value).ok_or_else(bad)?,
        "gap" => node.gap = parse_dp(value).ok_or_else(bad)?,
        other => return Err(PatchError::UnknownProperty(other.to_string())),
    }
    Ok(())
}

/// Applies all operations to a copy of `spec`. The input is never modified, so a failed
/// patch leaves the original spec unchanged (all-or-nothing, 03 section 6.3).
pub fn apply_patch(spec: &UiSpec, ops: &[PatchOp]) -> Result<UiSpec, PatchError> {
    let mut next = spec.clone();
    for op in ops {
        match op {
            PatchOp::SetProp {
                target,
                name,
                value,
                expect,
            } => {
                let node = find_mut(&mut next.root, target)
                    .ok_or_else(|| PatchError::TargetNotFound(target.clone()))?;
                let actual = read_prop(node, name)?;
                if let Some(expected) = expect {
                    if *expected != actual {
                        return Err(PatchError::Conflict {
                            target: target.clone(),
                            name: name.clone(),
                            expected: expected.clone(),
                            actual,
                        });
                    }
                }
                write_prop(node, name, value)?;
            }
        }
    }
    Ok(next)
}

/// Reads a property in textual form, for display and for `expect` checks.
pub fn property_text(spec: &UiSpec, target: &str, name: &str) -> Option<String> {
    let mut root = spec.root.clone();
    let node = find_mut(&mut root, target)?;
    read_prop(node, name).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_root() -> Node {
        Node::new("main", Kind::Row)
            .with_padding(48.0)
            .with_children(vec![
                Node::new("sidebar", Kind::Leaf)
                    .with_width(Length::Dp(240.0))
                    .with_min_width(240.0),
                Node::new("content", Kind::Leaf)
                    .with_width(Length::Fill)
                    .with_min_width(400.0),
            ])
    }

    #[test]
    fn row_requires_sum_of_child_min_widths_plus_padding() {
        assert_eq!(required_min_width(&sample_root()), 736.0);
    }

    #[test]
    fn empty_row_requires_only_padding() {
        let n = Node::new("empty", Kind::Row).with_padding(10.0);
        assert_eq!(required_min_width(&n), 20.0);
    }

    #[test]
    fn check_window_reports_conflict_below_minimum() {
        let d = check_window(&sample_root(), 720.0);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].code, "LAYOUT_CONFLICT");
        assert_eq!(d[0].actual, 736.0);
        assert_eq!(d[0].expected, 720.0);
        assert_eq!(d[0].overflow, 16.0);
    }

    #[test]
    fn check_window_is_empty_when_window_fits() {
        assert!(check_window(&sample_root(), 800.0).is_empty());
    }

    #[test]
    fn diagnostic_json_has_stable_code_field() {
        let d = check_window(&sample_root(), 720.0);
        let v: serde_json::Value = serde_json::from_str(&d[0].to_json()).unwrap();
        assert_eq!(v["code"], "LAYOUT_CONFLICT");
    }

    #[test]
    fn widening_sidebar_min_width_changes_required_width() {
        let mut root = sample_root();
        find_mut(&mut root, "sidebar").unwrap().min_width = 280.0;
        assert_eq!(required_min_width(&root), 776.0);
    }

    #[test]
    fn pseudo_long_label_reports_conflict() {
        // Simulates a long translated label that raises the content minimum width.
        let mut root = sample_root();
        find_mut(&mut root, "content").unwrap().min_width = 560.0;
        let d = check_window(&root, 800.0);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].overflow, 896.0 - 800.0);
    }

    #[test]
    fn window_minimum_width_is_enforced_through_root_min_width() {
        let mut root = Node::new("window", Kind::Window)
            .with_min_width(800.0)
            .with_children(vec![sample_root()]);
        // Window minimum is 800 dp; layout requires 736 dp, so the minimum dominates.
        assert_eq!(required_min_width(&root), 800.0);
        // Root minimum above the window width is a conflict.
        root.min_width = 900.0;
        let d = check_window(&root, 800.0);
        assert_eq!(d[0].overflow, 100.0);
    }
}
