mod helpers;

use helpers::TdoTest;

#[test]
fn create_prints_id_and_creates_file() {
    let t = TdoTest::new();
    let id = t.run_ok(&["hello", "world"]);

    // ID should be 4 hex chars
    assert_eq!(id.len(), 4, "expected 4-char hex id, got: '{id}'");
    assert!(
        id.chars().all(|c| c.is_ascii_hexdigit()),
        "id should be hex: '{id}'"
    );

    // A .md file should exist in the directory
    let files = t.files();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with(".md"));
    assert!(files[0].starts_with(&id));
    assert!(files[0].contains("hello-world"));
}

#[test]
fn list_shows_created_todo() {
    let t = TdoTest::new();
    let id = t.run_ok(&["buy milk"]);
    let list = t.run_ok(&["--list"]);
    assert!(list.contains(&id));
    assert!(list.contains("buy milk"));
}

#[test]
fn list_hides_done_todos() {
    let t = TdoTest::new();
    let id = t.run_ok(&["task one"]);
    t.run_ok(&["task two"]);

    // Mark first as done
    t.run_ok(&["--done", &id]);

    // Regular list should only show task two
    let list = t.run_ok(&["--list"]);
    assert!(!list.contains(&id), "done todo should be hidden");
    assert!(list.contains("task two"));
}

#[test]
fn list_all_shows_done_todos() {
    let t = TdoTest::new();
    let id = t.run_ok(&["task one"]);
    t.run_ok(&["--done", &id]);

    let list = t.run_ok(&["--list", "--all"]);
    assert!(list.contains(&id));
    assert!(list.contains("[done]"));
}

#[test]
fn done_marks_status() {
    let t = TdoTest::new();
    let id = t.run_ok(&["do the thing"]);
    t.run_ok(&["--done", &id]);

    // Verify the file contains status: done
    let files = t.files();
    let path = t.dir.path().join(&files[0]);
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("status: done"));
}

#[test]
fn delete_removes_file() {
    let t = TdoTest::new();
    let id = t.run_ok(&["delete me"]);
    assert_eq!(t.files().len(), 1);

    // Non-interactive delete (piped stdin) should skip confirmation
    t.run_ok(&["--delete", &id]);
    assert_eq!(t.files().len(), 0);
}

#[test]
fn edit_non_interactive_title() {
    let t = TdoTest::new();
    let id = t.run_ok(&["original title"]);
    t.run_ok(&["--edit", &id, "--title", "updated title"]);

    let files = t.files();
    let path = t.dir.path().join(&files[0]);
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("title: updated title"));
}

#[test]
fn edit_non_interactive_body() {
    let t = TdoTest::new();
    let id = t.run_ok(&["some task"]);
    t.run_ok(&["--edit", &id, "--body", "detailed notes here"]);

    let files = t.files();
    let path = t.dir.path().join(&files[0]);
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("detailed notes here"));
}

#[test]
fn unknown_id_fails() {
    let t = TdoTest::new();
    t.run_err(&["--done", "ffff"]);
}

#[test]
fn no_args_non_interactive_lists_open() {
    let t = TdoTest::new();
    let id = t.run_ok(&["auto listed"]);

    // Running with no args in a non-interactive context (piped)
    let list = t.run_ok(&[]);
    assert!(list.contains(&id));
    assert!(list.contains("auto listed"));
}

#[test]
fn multiple_creates_unique_ids() {
    let t = TdoTest::new();
    let id1 = t.run_ok(&["first"]);
    let id2 = t.run_ok(&["second"]);
    let id3 = t.run_ok(&["third"]);
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}
