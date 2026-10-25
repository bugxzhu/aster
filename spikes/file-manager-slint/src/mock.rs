#[derive(Debug, Clone, PartialEq)]
pub struct MockFile {
    pub name: String,
    pub kind: String,
    pub modified: String,
}

pub fn files() -> Vec<MockFile> {
    let rows: [(&str, &str, &str); 24] = [
        ("Quarterly report.pdf", "PDF", "2026-10-02"),
        ("Budget 2026.xlsx", "Spreadsheet", "2026-09-28"),
        ("Project notes.md", "Markdown", "2026-10-20"),
        ("Architecture.png", "Image", "2026-10-11"),
        ("Meeting recording.mp4", "Video", "2026-08-15"),
        ("report-draft-v2.docx", "Document", "2026-10-22"),
        ("Invoice 0042.pdf", "PDF", "2026-07-30"),
        ("Logo final.svg", "Image", "2026-06-12"),
        ("todo.txt", "Text", "2026-10-24"),
        ("Roadmap.pptx", "Presentation", "2026-09-05"),
        (
            "Long file name that must be truncated without breaking layout.txt",
            "Text",
            "2026-10-01",
        ),
        ("Resume.pdf", "PDF", "2026-05-19"),
        ("Backup 2026-10-01.zip", "Archive", "2026-10-01"),
        ("screenshot 1.png", "Image", "2026-10-23"),
        ("screenshot 2.png", "Image", "2026-10-23"),
        ("config.toml", "Config", "2026-10-18"),
        ("main.rs", "Source", "2026-10-21"),
        ("lib.rs", "Source", "2026-10-21"),
        ("Contract signed.pdf", "PDF", "2026-04-02"),
        ("Holiday photos", "Folder", "2026-03-14"),
        ("Music", "Folder", "2026-02-20"),
        ("Empty folder", "Folder", "2026-01-09"),
        ("Notes 2025.txt", "Text", "2025-12-31"),
        ("Unicode \u{6587}\u{4EF6}\u{540D}.txt", "Text", "2026-10-10"),
    ];
    rows.iter()
        .map(|(n, k, m)| MockFile {
            name: n.to_string(),
            kind: k.to_string(),
            modified: m.to_string(),
        })
        .collect()
}

pub fn filter(files: &[MockFile], query: &str) -> Vec<MockFile> {
    let q = query.to_lowercase();
    files
        .iter()
        .filter(|f| f.name.to_lowercase().contains(&q))
        .cloned()
        .collect()
}
