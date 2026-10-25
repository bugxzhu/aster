use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Violation {
    pub path: String,
    pub line: usize,
    pub text: String,
}

/// Returns true if the string contains a CJK ideograph or CJK punctuation/fullwidth form.
pub fn contains_cjk(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(c,
            '\u{3000}'..='\u{303F}'   // CJK symbols and punctuation
            | '\u{3400}'..='\u{4DBF}' // CJK extension A
            | '\u{4E00}'..='\u{9FFF}' // CJK unified ideographs
            | '\u{FF00}'..='\u{FFEF}' // fullwidth forms
        )
    })
}

/// Files whose content is subject to the English-only policy (06 section 3.0).
pub fn should_scan(path: &Path) -> bool {
    let s = path.to_string_lossy();
    if s.starts_with("target/")
        || s.starts_with("docs/")
        || s.starts_with("locales/")
        || s.starts_with("i18n/")
    {
        return false;
    }
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs") | Some("slint") | Some("toml")
    )
}

/// Scan one file's text and return every line containing CJK characters.
pub fn scan_text(path: &str, text: &str) -> Vec<Violation> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| contains_cjk(line))
        .map(|(i, line)| Violation {
            path: path.to_string(),
            line: i + 1,
            text: line.trim().to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // CJK sample written as escape sequences, so this file contains no CJK characters.
    const ZH: &str = "\u{4E2D}\u{6587}";

    #[test]
    fn contains_cjk_detects_han_characters() {
        assert!(contains_cjk(&format!("// comment {ZH}")));
    }

    #[test]
    fn contains_cjk_ignores_ascii() {
        assert!(!contains_cjk("// plain English comment"));
    }

    #[test]
    fn scan_text_reports_line_of_violation() {
        let text = format!("fn a() {{}}\n// {ZH}\n");
        let v = scan_text("crates/x/src/lib.rs", &text);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].line, 2);
    }

    #[test]
    fn should_scan_skips_locale_data_and_docs() {
        assert!(!should_scan(Path::new("locales/zh-CN.toml")));
        assert!(!should_scan(Path::new("docs/adr/README.md")));
        assert!(!should_scan(Path::new("target/debug/build.rs")));
    }

    #[test]
    fn should_scan_includes_rust_and_slint_sources() {
        assert!(should_scan(Path::new("crates/aster-diag/src/lib.rs")));
        assert!(should_scan(Path::new(
            "spikes/file-manager-slint/ui/app.slint"
        )));
    }
}
