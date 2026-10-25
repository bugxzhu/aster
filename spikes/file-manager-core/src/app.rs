use crate::mock::{self, MockFile};
use crate::strings::{self, Strings};

/// Locale packs supported by the spike, in cycling order.
pub const LOCALES: [&str; 2] = ["en", "zh-CN"];

/// Application state. Only the business logic lives here (02 section 8: state is changed by actions).
#[derive(Debug)]
pub struct AppState {
    all: Vec<MockFile>,
    query: String,
    selected: Option<String>,
    dark: bool,
    locale_index: usize,
    strings: Strings,
}

impl AppState {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            all: mock::files(),
            query: String::new(),
            selected: None,
            dark: false,
            locale_index: 0,
            strings: strings::load(LOCALES[0])?,
        })
    }

    pub fn strings(&self) -> &Strings {
        &self.strings
    }

    pub fn locale_tag(&self) -> &'static str {
        LOCALES[self.locale_index]
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    /// Files matching the current query, in mock-data order.
    pub fn rows(&self) -> Vec<MockFile> {
        mock::filter(&self.all, &self.query)
    }

    pub fn search(&mut self, query: &str) {
        self.query = query.to_string();
    }

    pub fn select(&mut self, name: &str) {
        self.selected = Some(name.to_string());
    }

    pub fn detail_text(&self) -> String {
        match &self.selected {
            Some(name) => name.clone(),
            None => self.strings.no_selection.clone(),
        }
    }

    pub fn is_dark(&self) -> bool {
        self.dark
    }

    pub fn toggle_theme(&mut self) {
        self.dark = !self.dark;
    }

    /// Label of the theme button: it names the theme that a click switches to.
    pub fn theme_label(&self) -> &str {
        if self.dark {
            &self.strings.theme_light
        } else {
            &self.strings.theme_dark
        }
    }

    /// Switches to the next locale pack. Fails without changing state if the pack cannot be loaded.
    pub fn toggle_language(&mut self) -> Result<(), String> {
        let next = (self.locale_index + 1) % LOCALES.len();
        self.strings = strings::load(LOCALES[next])?;
        self.locale_index = next;
        Ok(())
    }
}
