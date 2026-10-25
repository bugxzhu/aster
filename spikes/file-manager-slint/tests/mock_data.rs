use file_manager_slint::mock::{self};

#[test]
fn mock_data_has_files_for_list() {
    assert!(
        mock::files().len() >= 20,
        "list needs enough rows to scroll"
    );
}

#[test]
fn filter_matches_name_case_insensitively() {
    let files = mock::files();
    let hits = mock::filter(&files, "REPORT");
    assert!(hits
        .iter()
        .all(|f| f.name.to_lowercase().contains("report")));
    assert!(!hits.is_empty());
}

#[test]
fn empty_query_returns_all_files() {
    let files = mock::files();
    assert_eq!(mock::filter(&files, "").len(), files.len());
}

#[test]
fn empty_list_shows_empty_state() {
    // Review Focus item 4: no results must be representable without panicking.
    let files = mock::files();
    assert!(mock::filter(&files, "zzz-no-such-file").is_empty());
}

#[test]
fn unicode_file_name_is_matched() {
    // Review Focus item: non-ASCII names. Written as escapes to keep this file ASCII.
    let files = mock::files();
    let query = "\u{6587}\u{4EF6}";
    assert!(mock::filter(&files, query)
        .iter()
        .any(|f| f.name.contains(query)));
}
