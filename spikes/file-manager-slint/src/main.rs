use file_manager_slint::{mock, strings};
use slint::{ModelRc, SharedString, VecModel};

slint::include_modules!();

fn to_rows(files: &[mock::MockFile]) -> ModelRc<FileRow> {
    let rows: Vec<FileRow> = files
        .iter()
        .map(|f| FileRow {
            name: SharedString::from(f.name.as_str()),
            kind: SharedString::from(f.kind.as_str()),
            modified: SharedString::from(f.modified.as_str()),
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}

fn apply_strings(ui: &MainWindow, s: &strings::Strings) {
    ui.set_lbl_new_folder(s.new_folder.as_str().into());
    ui.set_lbl_refresh(s.refresh.as_str().into());
    ui.set_lbl_theme_dark(s.theme_dark.as_str().into());
    ui.set_lbl_theme_light(s.theme_light.as_str().into());
    ui.set_lbl_search(s.search_placeholder.as_str().into());
    ui.set_lbl_all_files(s.all_files.as_str().into());
    ui.set_lbl_recent(s.recent.as_str().into());
    ui.set_lbl_favorites(s.favorites.as_str().into());
    ui.set_lbl_no_selection(s.no_selection.as_str().into());
    ui.set_lbl_language(s.language.as_str().into());
}

fn main() -> Result<(), slint::PlatformError> {
    let all = mock::files();
    let ui = MainWindow::new()?;
    ui.set_files(to_rows(&all));

    // Start in English; fall back to the built-in labels if the pack is missing.
    let locales = ["en", "zh-CN"];
    let mut current = 0usize;
    if let Ok(s) = strings::load(locales[current]) {
        apply_strings(&ui, &s);
    }

    let weak = ui.as_weak();
    let source = all.clone();
    ui.on_search_changed(move |q| {
        if let Some(ui) = weak.upgrade() {
            let hits = mock::filter(&source, &q);
            ui.set_files(to_rows(&hits));
        }
    });

    let weak = ui.as_weak();
    ui.on_theme_toggled(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_dark(!ui.get_dark());
        }
    });

    // Runtime language switch: no restart needed (02 section 9.1).
    let weak = ui.as_weak();
    ui.on_language_toggled(move || {
        if let Some(ui) = weak.upgrade() {
            current = (current + 1) % locales.len();
            if let Ok(s) = strings::load(locales[current]) {
                apply_strings(&ui, &s);
            }
        }
    });

    ui.run()
}
