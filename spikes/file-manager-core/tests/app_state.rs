use file_manager_core::app::AppState;

#[test]
fn search_filters_rows_and_empty_query_restores_them() {
    let mut app = AppState::new().unwrap();
    let all = app.rows().len();
    app.search("report");
    assert!(app.rows().iter().all(|f| f.name.to_lowercase().contains("report")));
    app.search("");
    assert_eq!(app.rows().len(), all);
}

#[test]
fn detail_text_shows_no_selection_then_selected_name() {
    let mut app = AppState::new().unwrap();
    assert_eq!(app.detail_text(), app.strings().no_selection);
    app.select("todo.txt");
    assert_eq!(app.detail_text(), "todo.txt");
}

#[test]
fn theme_toggle_flips_mode_and_label() {
    let mut app = AppState::new().unwrap();
    let dark_label = app.strings().theme_dark.clone();
    assert_eq!(app.theme_label(), dark_label);
    app.toggle_theme();
    assert!(app.is_dark());
    assert_eq!(app.theme_label(), app.strings().theme_light.as_str());
}

#[test]
fn language_toggle_changes_strings_and_cycles_back() {
    let mut app = AppState::new().unwrap();
    assert_eq!(app.locale_tag(), "en");
    let english = app.strings().new_folder.clone();
    app.toggle_language().unwrap();
    assert_eq!(app.locale_tag(), "zh-CN");
    assert_ne!(app.strings().new_folder, english);
    app.toggle_language().unwrap();
    assert_eq!(app.strings().new_folder, english);
}
