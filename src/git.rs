use std::path::Path;

use crate::error::{Error, Result};

pub struct GitAuthor {
    pub name: String,
    pub email: Option<String>,
}

pub fn discover_repo(start_dir: &Path) -> Result<git2::Repository> {
    git2::Repository::discover(start_dir).map_err(|e| {
        Error::validation(format!(
            "not a git repository (or any parent): {}",
            e.message()
        ))
    })
}

pub fn repo_root(repo: &git2::Repository) -> Result<&Path> {
    repo.workdir()
        .ok_or_else(|| Error::validation("repository is bare (no working directory)"))
}

pub fn get_author(repo: &git2::Repository) -> Result<GitAuthor> {
    let config = repo.config()?;

    let entry = config.get_entry("user.name").map_err(|_| {
        Error::validation(
            "git user.name is not configured. Set it with: git config user.name \"Your Name\"",
        )
    })?;
    let name = entry.value().map(|v| v.to_string()).map_err(|_| {
        Error::validation(
            "git user.name is not configured. Set it with: git config user.name \"Your Name\"",
        )
    })?;

    let email = config
        .get_entry("user.email")
        .ok()
        .and_then(|entry| entry.value().ok().map(|v| v.to_string()));

    Ok(GitAuthor { name, email })
}

pub fn current_branch(repo: &git2::Repository) -> Result<String> {
    let head = repo.head()?;
    let name = head
        .shorthand()
        .map_err(|_| Error::validation("HEAD does not point to a named branch"))?;
    Ok(name.to_string())
}

pub fn create_and_checkout_branch(repo: &mut git2::Repository, branch_name: &str) -> Result<()> {
    let head = repo.head()?;
    let oid = head
        .target()
        .ok_or_else(|| Error::validation("HEAD does not point to a commit"))?;
    let commit = repo.find_commit(oid)?;

    repo.branch(branch_name, &commit, false)?;
    let ref_name = format!("refs/heads/{branch_name}");
    let ref_name_obj = repo.find_reference(&ref_name)?;
    let obj = ref_name_obj.peel(git2::ObjectType::Commit)?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&ref_name)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_repo(dir: &Path) -> git2::Repository {
        let repo = git2::Repository::init(dir).expect("init repo");
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

        repo
    }

    #[test]
    fn test_discover_repo_valid() {
        let tmp = TempDir::new().expect("temp dir");
        let repo = init_repo(tmp.path());
        drop(repo);

        let found = discover_repo(tmp.path()).expect("should discover");
        assert!(found.workdir().is_some());
    }

    #[test]
    fn test_discover_repo_invalid() {
        let tmp = TempDir::new().expect("temp dir");
        let result = discover_repo(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_author() {
        let tmp = TempDir::new().expect("temp dir");
        let repo = init_repo(tmp.path());
        let author = get_author(&repo).expect("get author");
        assert_eq!(author.name, "Test User");
        assert_eq!(author.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_current_branch() {
        let tmp = TempDir::new().expect("temp dir");
        let repo = init_repo(tmp.path());
        let branch = current_branch(&repo).expect("current branch");
        assert!(!branch.is_empty());
    }

    #[test]
    fn test_create_and_checkout_branch() {
        let tmp = TempDir::new().expect("temp dir");
        let mut repo = init_repo(tmp.path());
        create_and_checkout_branch(&mut repo, "feature-test").expect("create branch");
        let branch = current_branch(&repo).expect("current branch");
        assert_eq!(branch, "feature-test");
    }
}
