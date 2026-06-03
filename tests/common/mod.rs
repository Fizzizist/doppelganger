use std::path::Path;
use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;
use tempfile::TempDir;

pub fn setup_repo() -> (TempDir, git2::Repository) {
    let tmp = TempDir::new().expect("temp dir");
    let repo = git2::Repository::init(tmp.path()).expect("init repo");
    let mut config = repo.config().expect("get config");
    config.set_str("user.name", "Test User").expect("set name");
    config
        .set_str("user.email", "test@example.com")
        .expect("set email");

    let sig = repo.signature().expect("signature");
    let tree_id = {
        let mut index = repo.index().expect("index");
        let oid = index.write_tree().expect("write tree");
        repo.find_tree(oid).expect("find tree").id()
    };
    let tree = repo.find_tree(tree_id).expect("find tree");
    repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
        .expect("commit");
    drop(tree);

    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
        .expect("checkout head");

    (tmp, repo)
}

pub fn dg_command() -> Command {
    Command::cargo_bin("doppelganger").expect("find binary")
}

pub fn dg_command_in_repo(repo_path: &Path) -> Command {
    let mut cmd = dg_command();
    cmd.current_dir(repo_path);
    cmd
}
