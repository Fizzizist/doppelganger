pub fn discover_repo() -> crate::error::Result<git2::Repository> {
    git2::Repository::discover(".").map_err(|_| crate::error::Error::NoRepository)
}

pub fn repo_root(repo: &git2::Repository) -> crate::error::Result<std::path::PathBuf> {
    repo.workdir()
        .map(|p| p.to_path_buf())
        .ok_or(crate::error::Error::NoRepository)
}

pub fn current_branch(repo: &git2::Repository) -> crate::error::Result<String> {
    if repo.head_detached()? {
        return Err(crate::error::Error::DetachedHead);
    }
    let head = repo.head()?;
    let shorthand = head.shorthand()?;
    Ok(shorthand.to_string())
}

pub fn remote_url(repo: &git2::Repository) -> crate::error::Result<String> {
    let remote = repo
        .find_remote("origin")
        .map_err(|_| crate::error::Error::NoRemote)?;
    let url = remote.url().map_err(|_| crate::error::Error::NoRemote)?;
    Ok(url.to_string())
}
