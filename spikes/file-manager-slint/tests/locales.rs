use file_manager_slint::strings;

#[test]
fn english_pack_loads_with_all_keys_filled() {
    let s = strings::load("en").expect("en pack loads");
    assert!(s.values().iter().all(|v| !v.is_empty()));
}

#[test]
fn zh_cn_pack_loads_with_all_keys_filled() {
    let s = strings::load("zh-CN").expect("zh-CN pack loads");
    assert!(s.values().iter().all(|v| !v.is_empty()));
}

#[test]
fn zh_cn_translates_every_key_that_english_defines() {
    let en = strings::load("en").unwrap();
    let zh = strings::load("zh-CN").unwrap();
    // Every key must differ from English; identical values would mean an untranslated key.
    for (a, b) in en.values().iter().zip(zh.values().iter()) {
        assert_ne!(a, b, "untranslated key: {a}");
    }
}

#[test]
fn unknown_locale_is_an_error_not_a_panic() {
    assert!(strings::load("xx-NO-SUCH").is_err());
}
