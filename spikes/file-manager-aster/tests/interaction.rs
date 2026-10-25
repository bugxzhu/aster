//! Drives the real Slint instance through its callbacks (no pixels involved) and reads back
//! bound properties. Covers Review Focus items 4 (empty result) and the language/theme switches.
//! Visual checks (IME, DPI, minimum window size) are NOT covered here; see the manual check record.

use aster_backend_slint::prop_name;
use file_manager_aster::{create, spec::file_manager_spec};
use slint_interpreter::{ComponentInstance, Value};

fn string_prop(instance: &ComponentInstance, key: &str) -> String {
    match instance
        .get_property(&prop_name(key))
        .expect("property exists")
    {
        Value::String(s) => s.to_string(),
        other => panic!("{key} is not a string: {other:?}"),
    }
}

fn row_count(instance: &ComponentInstance) -> usize {
    match instance.get_property("files").expect("files exists") {
        Value::Model(m) => {
            use slint::Model;
            m.row_count()
        }
        other => panic!("files is not a model: {other:?}"),
    }
}

fn call_text(instance: &ComponentInstance, name: &str, arg: &str) {
    instance
        .invoke(name, &[Value::String(arg.into())])
        .unwrap_or_else(|e| panic!("invoke {name}: {e:?}"));
}

fn call_unit(instance: &ComponentInstance, name: &str) {
    instance
        .invoke(name, &[])
        .unwrap_or_else(|e| panic!("invoke {name}: {e:?}"));
}

#[test]
fn search_filters_then_empty_result_shows_zero_rows() {
    i_slint_backend_testing::init_no_event_loop();
    let (session, _app) = create(&file_manager_spec()).expect("window created");
    let all = row_count(session.instance());
    assert!(all >= 20);

    call_text(session.instance(), "search_changed", "report");
    let hits = row_count(session.instance());
    assert!(
        hits > 0 && hits < all,
        "filter had no effect: {hits} of {all}"
    );

    call_text(session.instance(), "search_changed", "zzz-no-such-file");
    assert_eq!(
        row_count(session.instance()),
        0,
        "empty result must produce an empty list"
    );
    assert_eq!(string_prop(session.instance(), "query"), "zzz-no-such-file");
}

#[test]
fn selecting_a_file_updates_the_detail_text() {
    i_slint_backend_testing::init_no_event_loop();
    let (session, _app) = create(&file_manager_spec()).expect("window created");
    let before = string_prop(session.instance(), "detail_text");
    call_text(session.instance(), "file_selected", "todo.txt");
    assert_eq!(string_prop(session.instance(), "detail_text"), "todo.txt");
    assert_ne!(before, "todo.txt");
}

#[test]
fn theme_and_language_switches_update_bound_labels_without_restart() {
    i_slint_backend_testing::init_no_event_loop();
    let (session, _app) = create(&file_manager_spec()).expect("window created");
    let en_new_folder = string_prop(session.instance(), "ui.new_folder");
    let theme_before = string_prop(session.instance(), "ui.theme_label");

    call_unit(session.instance(), "theme_toggled");
    assert_ne!(
        string_prop(session.instance(), "ui.theme_label"),
        theme_before
    );

    call_unit(session.instance(), "language_toggled");
    let zh_new_folder = string_prop(session.instance(), "ui.new_folder");
    assert_ne!(
        zh_new_folder, en_new_folder,
        "language switch must change bound text"
    );

    call_unit(session.instance(), "language_toggled");
    assert_eq!(
        string_prop(session.instance(), "ui.new_folder"),
        en_new_folder
    );
}
