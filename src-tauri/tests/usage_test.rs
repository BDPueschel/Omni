use omni_lib::usage::{
    clear_usage_with, get_frequent_with, get_usage_with, record_usage_with, test_connection,
};

#[test]
fn test_record_and_retrieve_usage() {
    let conn = test_connection();
    record_usage_with(&conn, "notepad", r"C:\Windows\notepad.exe", "Apps", "Notepad");

    let results = get_usage_with(&conn, "notepad");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, r"C:\Windows\notepad.exe");
    assert_eq!(results[0].1, "Apps");
    assert_eq!(results[0].2, "Notepad");
    assert_eq!(results[0].3, 1);
}

#[test]
fn test_count_increments_on_repeated_selection() {
    let conn = test_connection();
    record_usage_with(&conn, "notepad", r"C:\Windows\notepad.exe", "Apps", "Notepad");
    record_usage_with(&conn, "notepad", r"C:\Windows\notepad.exe", "Apps", "Notepad");
    record_usage_with(&conn, "notepad", r"C:\Windows\notepad.exe", "Apps", "Notepad");

    let results = get_usage_with(&conn, "notepad");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].3, 3);
}

#[test]
fn test_query_normalization() {
    let conn = test_connection();
    record_usage_with(&conn, "  NotePad  ", r"C:\Windows\notepad.exe", "Apps", "Notepad");

    // Should find it with different casing/whitespace
    let results = get_usage_with(&conn, "notepad");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].3, 1);

    // Record again with different casing — should increment
    record_usage_with(&conn, "NOTEPAD", r"C:\Windows\notepad.exe", "Apps", "Notepad");
    let results = get_usage_with(&conn, "notepad");
    assert_eq!(results[0].3, 2);
}

#[test]
fn test_get_frequent_returns_items_with_count_gte_3() {
    let conn = test_connection();

    // Item with count=2 should NOT appear
    record_usage_with(&conn, "calc", r"C:\Windows\calc.exe", "Apps", "Calculator");
    record_usage_with(&conn, "calc", r"C:\Windows\calc.exe", "Apps", "Calculator");

    // Item with count=3 should appear
    record_usage_with(&conn, "notepad", r"C:\Windows\notepad.exe", "Apps", "Notepad");
    record_usage_with(&conn, "notepad", r"C:\Windows\notepad.exe", "Apps", "Notepad");
    record_usage_with(&conn, "notepad", r"C:\Windows\notepad.exe", "Apps", "Notepad");

    let frequent = get_frequent_with(&conn, 10);
    assert_eq!(frequent.len(), 1);
    assert_eq!(frequent[0].0, r"C:\Windows\notepad.exe");
    assert_eq!(frequent[0].3, 3);
}

#[test]
fn test_get_frequent_respects_limit() {
    let conn = test_connection();

    // Create 3 items each with count >= 3
    for _ in 0..5 {
        record_usage_with(&conn, "a", r"C:\a.exe", "Apps", "A");
        record_usage_with(&conn, "b", r"C:\b.exe", "Apps", "B");
        record_usage_with(&conn, "c", r"C:\c.exe", "Apps", "C");
    }

    let frequent = get_frequent_with(&conn, 2);
    assert_eq!(frequent.len(), 2);
}

#[test]
fn test_clear_usage_empties_data() {
    let conn = test_connection();
    record_usage_with(&conn, "notepad", r"C:\Windows\notepad.exe", "Apps", "Notepad");
    record_usage_with(&conn, "calc", r"C:\Windows\calc.exe", "Apps", "Calculator");

    clear_usage_with(&conn);

    let results = get_usage_with(&conn, "notepad");
    assert!(results.is_empty());

    let frequent = get_frequent_with(&conn, 10);
    assert!(frequent.is_empty());
}

#[test]
fn test_multiple_results_for_same_query() {
    let conn = test_connection();
    record_usage_with(&conn, "note", r"C:\Windows\notepad.exe", "Apps", "Notepad");
    record_usage_with(&conn, "note", r"C:\Windows\notepad.exe", "Apps", "Notepad");
    record_usage_with(&conn, "note", r"C:\docs\notes.txt", "Files", "notes.txt");

    let results = get_usage_with(&conn, "note");
    assert_eq!(results.len(), 2);
    // Higher count first
    assert_eq!(results[0].0, r"C:\Windows\notepad.exe");
    assert_eq!(results[0].3, 2);
    assert_eq!(results[1].0, r"C:\docs\notes.txt");
    assert_eq!(results[1].3, 1);
}

#[test]
fn test_title_updates_on_reselection() {
    let conn = test_connection();
    record_usage_with(&conn, "note", r"C:\Windows\notepad.exe", "Apps", "Old Title");
    record_usage_with(&conn, "note", r"C:\Windows\notepad.exe", "Apps", "New Title");

    let results = get_usage_with(&conn, "note");
    assert_eq!(results[0].2, "New Title");
    assert_eq!(results[0].3, 2);
}
