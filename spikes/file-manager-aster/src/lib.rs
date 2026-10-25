pub mod spec;

use aster_backend_slint::{LoadError, Session};
use aster_ui_model::UiSpec;
use file_manager_core::app::AppState;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Pushes every bound value from the business state into the window.
pub fn push_state(session: &Session, app: &AppState) -> Result<(), String> {
    let s = app.strings();
    session.set_text("ui.all_files", &s.all_files)?;
    session.set_text("ui.recent", &s.recent)?;
    session.set_text("ui.favorites", &s.favorites)?;
    session.set_text("ui.new_folder", &s.new_folder)?;
    session.set_text("ui.refresh", &s.refresh)?;
    session.set_text("ui.language", &s.language)?;
    session.set_text("ui.search_placeholder", &s.search_placeholder)?;
    session.set_text("ui.theme_label", app.theme_label())?;
    session.set_text("query", app.query())?;
    session.set_text("detail_text", &app.detail_text())?;
    push_rows(session, app)
}

fn push_rows(session: &Session, app: &AppState) -> Result<(), String> {
    let rows: Vec<[String; 3]> = app
        .rows()
        .into_iter()
        .map(|f| [f.name, f.kind, f.modified])
        .collect();
    session.set_file_rows("files", &rows)
}

/// Loads the spec into a window, connects callbacks to the business state and pushes the
/// initial values. Returns the session and the state for tests.
pub fn create(spec: &UiSpec) -> Result<(Rc<Session>, Rc<RefCell<AppState>>), String> {
    let app = Rc::new(RefCell::new(AppState::new()?));
    let session = Rc::new(aster_backend_slint::load(spec).map_err(describe)?);
    push_state(&session, &app.borrow())?;

    let wire_text = |name: &str, f: fn(&mut AppState, &str)| {
        let weak: Weak<Session> = Rc::downgrade(&session);
        let app = app.clone();
        session.on_text_callback(name, move |arg| {
            f(&mut app.borrow_mut(), arg);
            if let Some(s) = weak.upgrade() {
                let _ = push_state(&s, &app.borrow());
            }
        })
    };
    wire_text("search_changed", |a, q| a.search(q))?;
    wire_text("file_selected", |a, n| a.select(n))?;

    let wire_unit = |name: &str, f: fn(&mut AppState) -> Result<(), String>| {
        let weak: Weak<Session> = Rc::downgrade(&session);
        let app = app.clone();
        session.on_unit_callback(name, move || {
            if f(&mut app.borrow_mut()).is_ok() {
                if let Some(s) = weak.upgrade() {
                    let _ = push_state(&s, &app.borrow());
                }
            }
        })
    };
    wire_unit("theme_toggled", |a| {
        a.toggle_theme();
        Ok(())
    })?;
    wire_unit("language_toggled", |a| a.toggle_language())?;
    // Spike scope: these buttons are present in the layout but have no business action yet.
    wire_unit("new_folder_clicked", |_| Ok(()))?;
    wire_unit("refresh_clicked", |_| Ok(()))?;

    Ok((session, app))
}

fn describe(e: LoadError) -> String {
    format!("{e:?}")
}
