//! Headless geometry probe (feature `layout-probe`). Reads the rectangles that Slint's layout
//! engine computed for the elements generated from an Aster model, and compares them with the
//! declared model. Verification only: it uses the testing backend, which must not be linked into
//! production builds.

use crate::element_id;
use aster_ui_model::{Kind, Length, Node, UiSpec};
use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;
use slint_interpreter::ComponentInstance;

const TOLERANCE: f32 = 0.5;

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryDiagnostic {
    /// `GEOMETRY_MISMATCH`: the laid-out width differs from the declared `width`.
    /// `LAYOUT_OVERFLOW`: a child extends outside its parent's rectangle.
    /// `ELEMENT_NOT_FOUND`: the generated element could not be located.
    pub code: &'static str,
    pub node: String,
    pub property: &'static str,
    pub expected: f32,
    pub actual: f32,
    pub overflow: f32,
}

/// Rectangle of the element generated for `id`, in window coordinates.
pub fn rect_of(instance: &ComponentInstance, id: &str) -> Option<Rect> {
    let el =
        ElementHandle::find_by_element_id(instance, &format!("MainWindow::{}", element_id(id)))
            .next()?;
    let pos = el.absolute_position();
    let size = el.size();
    Some(Rect {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
    })
}

/// Checks every node of the spec against the real geometry.
pub fn check_geometry(instance: &ComponentInstance, spec: &UiSpec) -> Vec<GeometryDiagnostic> {
    let mut out = Vec::new();
    // Children of the window root are checked against the window itself.
    let window = Rect {
        x: 0.0,
        y: 0.0,
        width: instance.window().size().width as f32 / instance.window().scale_factor(),
        height: instance.window().size().height as f32 / instance.window().scale_factor(),
    };
    for child in &spec.root.children {
        check_node(instance, child, &window, &mut out);
    }
    out
}

fn check_node(
    instance: &ComponentInstance,
    node: &Node,
    parent: &Rect,
    out: &mut Vec<GeometryDiagnostic>,
) {
    let Some(rect) = rect_of(instance, &node.id) else {
        out.push(GeometryDiagnostic {
            code: "ELEMENT_NOT_FOUND",
            node: node.id.clone(),
            property: "element",
            expected: 1.0,
            actual: 0.0,
            overflow: 0.0,
        });
        return;
    };
    if let Length::Dp(declared) = node.width {
        if (rect.width - declared).abs() > TOLERANCE {
            out.push(GeometryDiagnostic {
                code: "GEOMETRY_MISMATCH",
                node: node.id.clone(),
                property: "width",
                expected: declared,
                actual: rect.width,
                overflow: 0.0,
            });
        }
    }
    let right_overflow = (rect.x + rect.width) - (parent.x + parent.width);
    let bottom_overflow = (rect.y + rect.height) - (parent.y + parent.height);
    let left_overflow = parent.x - rect.x;
    let top_overflow = parent.y - rect.y;
    let worst = [right_overflow, bottom_overflow, left_overflow, top_overflow]
        .into_iter()
        .fold(0.0_f32, f32::max);
    if worst > TOLERANCE {
        out.push(GeometryDiagnostic {
            code: "LAYOUT_OVERFLOW",
            node: node.id.clone(),
            property: "width",
            expected: parent.width,
            actual: rect.width,
            overflow: worst,
        });
    }
    if matches!(node.kind, Kind::Row | Kind::Column | Kind::Window) {
        for child in &node.children {
            check_node(instance, child, &rect, out);
        }
    }
}
