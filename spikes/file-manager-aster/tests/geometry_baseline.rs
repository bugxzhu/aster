//! Experiment 03 baseline: geometry of the real Slint layout produced from the Aster model.
//! Runs in its own process because the testing backend can be initialised once per process.

use file_manager_aster::{create, spec::file_manager_spec};
use i_slint_backend_testing::ElementHandle;
use slint::ComponentHandle;

fn width_of(instance: &impl ComponentHandle, id: &str) -> f32 {
    let el = ElementHandle::find_by_element_id(instance, id)
        .next()
        .unwrap_or_else(|| panic!("element {id} not found"));
    el.size().width
}

#[test]
fn baseline_sidebar_is_240_dp_and_content_fills_the_rest() {
    i_slint_backend_testing::init_no_event_loop();
    let spec = file_manager_spec();
    let (session, _app) = create(&spec).expect("spec loads and window is created");
    session.instance().show().expect("window shows");

    let window_width = session.instance().window().size().width as f32
        / session.instance().window().scale_factor();
    let sidebar = width_of(session.instance(), "MainWindow::sidebar");
    let content = width_of(session.instance(), "MainWindow::content");
    println!("window={window_width} sidebar={sidebar} content={content}");

    assert_eq!(sidebar, 240.0);
    assert_eq!(content, window_width - 240.0);
}
