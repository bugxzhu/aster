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
