mod helpers;

use helpers::TdoTest;

#[test]
fn create_prints_id_and_creates_file() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "hello", "world"]);

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
    let id = t.run_ok(&["add", "buy milk"]);
    let list = t.run_ok(&["list"]);
    assert!(list.contains(&id));
    assert!(list.contains("buy milk"));
}

#[test]
fn list_hides_done_todos() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "task one"]);
    t.run_ok(&["add", "task two"]);

    // Mark first as done
    t.run_ok(&["done", &id]);

    // Regular list should only show task two
    let list = t.run_ok(&["list"]);
    assert!(!list.contains(&id), "done todo should be hidden");
    assert!(list.contains("task two"));
}

#[test]
fn list_all_shows_done_todos() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "task one"]);
    t.run_ok(&["done", &id]);

    let list = t.run_ok(&["list", "--all"]);
    assert!(list.contains(&id));
    assert!(list.contains("[done]"));
}

#[test]
fn done_marks_status() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "do the thing"]);
    t.run_ok(&["done", &id]);

    // Verify the file contains status: done
    let files = t.files();
    let path = t.dir.path().join(&files[0]);
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("status: done"));
}

#[test]
fn reopen_marks_status() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "do the thing"]);
    t.run_ok(&["done", &id]);

    // Verify it's done
    let files = t.files();
    let path = t.dir.path().join(&files[0]);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("status: done"));

    // Reopen it
    t.run_ok(&["reopen", &id]);

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("status: open"));
}

#[test]
fn delete_requires_force_non_interactive() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "delete me"]);
    assert_eq!(t.files().len(), 1);

    // Without --force, non-interactive delete should fail
    let err = t.run_err(&["delete", &id]);
    assert!(err.contains("--force"), "should mention --force: {err}");
    assert_eq!(t.files().len(), 1, "file should not be deleted");
}

#[test]
fn delete_force_removes_file() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "delete me"]);
    assert_eq!(t.files().len(), 1);

    t.run_ok(&["delete", &id, "--force"]);
    assert_eq!(t.files().len(), 0);
}

#[test]
fn edit_non_interactive_body() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "some task"]);
    t.run_ok(&["edit", &id, "--body", "detailed notes here"]);

    let files = t.files();
    let path = t.dir.path().join(&files[0]);
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("detailed notes here"));
}

#[test]
fn edit_no_flags_non_interactive_fails() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "some task"]);

    // Non-interactive edit with no title/body should fail
    let err = t.run_err(&["edit", &id]);
    assert!(
        err.contains("non-interactively"),
        "should explain the error: {err}"
    );
}

#[test]
fn unknown_id_fails() {
    let t = TdoTest::new();
    t.run_err(&["done", "ffff"]);
}

#[test]
fn no_args_non_interactive_lists_open() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "auto listed"]);

    // Running with no args in a non-interactive context (piped)
    let list = t.run_ok(&[]);
    assert!(list.contains(&id));
    assert!(list.contains("auto listed"));
}

#[test]
fn multiple_creates_unique_ids() {
    let t = TdoTest::new();
    let id1 = t.run_ok(&["add", "first"]);
    let id2 = t.run_ok(&["add", "second"]);
    let id3 = t.run_ok(&["add", "third"]);
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn prefix_id_match() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "prefix test"]);
    // Use a 2-char prefix (should be unique with one todo)
    let prefix = &id[..2];

    t.run_ok(&["done", prefix]);

    let files = t.files();
    let path = t.dir.path().join(&files[0]);
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("status: done"));
}

#[test]
fn done_prints_feedback() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "feedback test"]);

    let output = t.run(&["done", &id]);
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("done:"), "should print feedback: {stderr}");
    assert!(stderr.contains(&id));
    assert!(stderr.contains("feedback test"));
}

#[test]
fn reopen_prints_feedback() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "reopen test"]);
    t.run_ok(&["done", &id]);

    let output = t.run(&["reopen", &id]);
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("reopened:"),
        "should print feedback: {stderr}"
    );
    assert!(stderr.contains(&id));
}

#[test]
fn delete_force_prints_feedback() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "delete feedback"]);

    let output = t.run(&["delete", &id, "--force"]);
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("deleted:"),
        "should print feedback: {stderr}"
    );
    assert!(stderr.contains(&id));
}

#[test]
fn add_with_body() {
    let t = TdoTest::new();
    let _id = t.run_ok(&["add", "my title", "--body", "line one\nline two"]);

    // Verify the file contains the body
    let files = t.files();
    assert_eq!(files.len(), 1);
    let content = std::fs::read_to_string(t.dir.path().join(&files[0])).unwrap();
    assert!(
        content.contains("title: my title"),
        "title should be set: {content}"
    );
    assert!(
        content.contains("line one\nline two"),
        "body should be set: {content}"
    );
}

#[test]
fn view_shows_todo_details() {
    let t = TdoTest::new();
    let id = t.run_ok(&["add", "fix", "login", "--body", "Check auth flow"]);
    let output = t.run_ok(&["view", &id]);
    assert!(output.contains(&id), "should contain ID: {output}");
    assert!(output.contains("fix login"), "should contain title: {output}");
    assert!(output.contains("status:   open"), "should contain status: {output}");
    assert!(output.contains("created:"), "should contain created: {output}");
    assert!(
        output.contains("Check auth flow"),
        "should contain body: {output}"
    );
}

#[test]
fn view_unknown_id_fails() {
    let t = TdoTest::new();
    let result = t.run_err(&["view", "ffff"]);
    assert!(
        result.contains("no todo found"),
        "should error on unknown id: {result}"
    );
}
