mod common;

use common::TestRepo;
use std::process::Command;

fn dg_bin() -> std::path::PathBuf {
    // assert_cmd resolves the built binary path; reuse its logic via env.
    assert_cmd::cargo::cargo_bin("dg")
}

// Regression test: a long-lived reader (the TUI opens/closes a connection each
// poll) plus concurrent writers must not produce turso file-lock errors. Before
// the fix, concurrent access returned "File is locked by another process".
#[test]
fn concurrent_writers_all_succeed() {
    let repo = TestRepo::new();
    let dg = dg_bin();

    // Seed an issue to comment on.
    let status = Command::new(&dg)
        .current_dir(&repo.path)
        .args(["issue", "create", "seed"])
        .status()
        .expect("spawn create");
    assert!(status.success(), "seed issue create should succeed");

    // Spawn several writers and readers concurrently against the same DB.
    let writers = 4;
    let per_writer = 10;
    let mut children = Vec::new();

    for w in 0..writers {
        for i in 0..per_writer {
            let dg = dg.clone();
            let path = repo.path.clone();
            children.push(std::thread::spawn(move || {
                Command::new(&dg)
                    .current_dir(&path)
                    .args(["issue", "comment", "1", &format!("w{w}c{i}")])
                    .status()
                    .expect("spawn comment")
                    .success()
            }));
        }
    }

    for r in 0..3 {
        let dg = dg.clone();
        let path = repo.path.clone();
        children.push(std::thread::spawn(move || {
            let mut ok = true;
            for _ in 0..10 {
                ok &= Command::new(&dg)
                    .current_dir(&path)
                    .args(["issue", "read", "1"])
                    .status()
                    .expect("spawn read")
                    .success();
            }
            let _ = r;
            ok
        }));
    }

    let all_ok = children
        .into_iter()
        .all(|h| h.join().expect("join child thread"));
    assert!(all_ok, "all concurrent dg invocations should succeed");

    // Every write must have landed.
    let output = Command::new(&dg)
        .current_dir(&repo.path)
        .args(["issue", "read", "1"])
        .output()
        .expect("spawn final read");
    assert!(output.status.success(), "final read should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("issue_comment_id").count();
    assert_eq!(
        count,
        writers * per_writer,
        "all concurrent writes should be persisted"
    );
}
