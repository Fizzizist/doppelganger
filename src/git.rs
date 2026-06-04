pub fn discover_repo() -> crate::error::Result<git2::Repository> {
    git2::Repository::discover(".").map_err(|_| crate::error::Error::NoRepository)
}

pub fn repo_root(repo: &git2::Repository) -> crate::error::Result<std::path::PathBuf> {
    repo.workdir()
        .map(|p| p.to_path_buf())
        .ok_or(crate::error::Error::NoRepository)
}

pub fn author_from_config(
    repo: &git2::Repository,
) -> crate::error::Result<(String, Option<String>)> {
    let config = repo.config()?;
    let name = config
        .get_string("user.name")
        .map_err(|_| crate::error::Error::MissingAuthorName)?;
    let email = config.get_string("user.email").ok();
    Ok((name, email))
}

pub fn current_branch(repo: &git2::Repository) -> crate::error::Result<String> {
    if repo.head_detached()? {
        return Err(crate::error::Error::DetachedHead);
    }
    let head = repo.head()?;
    let shorthand = head.shorthand()?;
    Ok(shorthand.to_string())
}
