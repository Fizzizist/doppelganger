use crate::error::{Error, Result};

pub struct RemoteIssue {
    pub remote_id: String,
    pub title: Option<String>,
    pub body: String,
    pub author: String,
    pub comments: Vec<RemoteComment>,
}

pub struct RemoteComment {
    pub author: String,
    pub body: String,
}

#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn fetch_issue(&self, issue_number: i64) -> Result<RemoteIssue>;
}

pub fn provider_from_config(
    config: &crate::config::Config,
    repo: &git2::Repository,
) -> Result<Box<dyn Provider>> {
    let github_config = config.github.as_ref().ok_or_else(|| {
        Error::RemoteSync(
            "remote token not configured; add a [github] section with token to your config"
                .to_string(),
        )
    })?;

    let url = crate::git::remote_url(repo)?;
    let (owner, repo_name) = github::parse_github_remote_url(&url)?;
    let provider = github::GitHubProvider::new(&github_config.token, &owner, &repo_name)?;
    Ok(Box::new(provider))
}

pub mod github;
