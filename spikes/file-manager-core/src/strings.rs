use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct Strings {
    pub new_folder: String,
    pub refresh: String,
    pub theme_dark: String,
    pub theme_light: String,
    pub search_placeholder: String,
    pub all_files: String,
    pub recent: String,
    pub favorites: String,
    pub no_selection: String,
    pub language: String,
}

#[derive(Debug, serde::Deserialize)]
struct LocaleFile {
    ui: Strings,
}

impl Strings {
    pub fn values(&self) -> Vec<&str> {
        vec![
            &self.new_folder,
            &self.refresh,
            &self.theme_dark,
            &self.theme_light,
            &self.search_placeholder,
            &self.all_files,
            &self.recent,
            &self.favorites,
            &self.no_selection,
            &self.language,
        ]
    }
}

/// Path of a locale pack in the repository `locales/` directory.
pub fn locale_path(tag: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("locales")
        .join(format!("{tag}.toml"))
}

pub fn load(tag: &str) -> Result<Strings, String> {
    let text = fs::read_to_string(locale_path(tag)).map_err(|e| e.to_string())?;
    toml::from_str::<LocaleFile>(&text)
        .map(|f| f.ui)
        .map_err(|e| e.to_string())
}
