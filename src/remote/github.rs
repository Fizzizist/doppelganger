use super::{Provider, RemoteComment, RemoteIssue};
use crate::error::{Error, Result};
use futures_util::StreamExt;

pub struct GitHubProvider {
    client: octocrab::Octocrab,
    owner: String,
    repo: String,
}

impl GitHubProvider {
    pub fn new(token: &str, owner: &str, repo: &str) -> crate::error::Result<Self> {
        let client = octocrab::Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| Error::Remote(format!("failed to build GitHub client: {e}")))?;
        Ok(Self {
            client,
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl Provider for GitHubProvider {
    async fn fetch_issue(&self, issue_number: i64) -> Result<RemoteIssue> {
        let issue = self
            .client
            .issues(&self.owner, &self.repo)
            .get(issue_number as u64)
            .await
            .map_err(|e| {
                Error::Remote(format!(
                    "failed to fetch GitHub issue #{issue_number} from {}/{}: {e}",
                    self.owner, self.repo
                ))
            })?;

        let author = issue.user.login.clone();
        let title = Some(issue.title);
        let body = issue.body.unwrap_or_default();

        let comments_page = self
            .client
            .issues(&self.owner, &self.repo)
            .list_comments(issue_number as u64)
            .send()
            .await
            .map_err(|e| {
                Error::Remote(format!(
                    "failed to fetch comments for GitHub issue #{issue_number} from {}/{}: {e}",
                    self.owner, self.repo
                ))
            })?;

        let comments = comments_page
            .into_stream(&self.client)
            .map(|comment| {
                let comment = comment.map_err(|e| Error::Remote(e.to_string()))?;
                let author = comment.user.login.clone();
                let body = comment.body.unwrap_or_default();
                Ok(RemoteComment { author, body })
            })
            .collect::<Vec<Result<RemoteComment>>>()
            .await;

        let comments = comments.into_iter().collect::<Result<Vec<_>>>()?;

        let remote_id = format!("gh:{}/{}#{}", self.owner, self.repo, issue_number);

        Ok(RemoteIssue {
            remote_id,
            title,
            body,
            author,
            comments,
        })
    }
}

pub fn parse_github_remote_url(url: &str) -> Result<(String, String)> {
    let url = url.trim();
    let path_part = if let Some(ssh) = url.strip_prefix("git@github.com:") {
        ssh.strip_suffix(".git").unwrap_or(ssh)
    } else if let Some(https) = url.strip_prefix("https://github.com/") {
        https.strip_suffix(".git").unwrap_or(https)
    } else if let Some(ssh) = url.strip_prefix("ssh://git@github.com/") {
        ssh.strip_suffix(".git").unwrap_or(ssh)
    } else if let Some(ssh_with_port) = url.strip_prefix("ssh://git@github.com:") {
        let after_port = ssh_with_port
            .split_once('/')
            .map(|(_, rest)| rest)
            .unwrap_or(ssh_with_port);
        after_port.strip_suffix(".git").unwrap_or(after_port)
    } else {
        return Err(Error::NoRemote);
    };

    let parts: Vec<&str> = path_part.split('/').collect();
    if parts.len() != 2 {
        return Err(Error::NoRemote);
    }
    let owner = parts[0].to_string();
    let repo = parts[1].to_string();
    Ok((owner, repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ssh_remote_url() {
        let url = "git@github.com:octocat/Hello-World.git";
        let (owner, repo) = parse_github_remote_url(url).unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_https_remote_url() {
        let url = "https://github.com/octocat/Hello-World.git";
        let (owner, repo) = parse_github_remote_url(url).unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_https_remote_url_without_git_suffix() {
        let url = "https://github.com/octocat/Hello-World";
        let (owner, repo) = parse_github_remote_url(url).unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_ssh_scheme_remote_url() {
        let url = "ssh://git@github.com/octocat/Hello-World.git";
        let (owner, repo) = parse_github_remote_url(url).unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_ssh_scheme_remote_url_without_git_suffix() {
        let url = "ssh://git@github.com/octocat/Hello-World";
        let (owner, repo) = parse_github_remote_url(url).unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_ssh_scheme_remote_url_with_port() {
        let url = "ssh://git@github.com:22/octocat/Hello-World.git";
        let (owner, repo) = parse_github_remote_url(url).unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_ssh_scheme_remote_url_with_port_no_git_suffix() {
        let url = "ssh://git@github.com:22/octocat/Hello-World";
        let (owner, repo) = parse_github_remote_url(url).unwrap();
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_non_github_remote_url() {
        let url = "https://gitlab.com/octocat/Hello-World.git";
        assert!(matches!(parse_github_remote_url(url), Err(Error::NoRemote)));
    }

    #[test]
    fn parse_invalid_github_remote_url() {
        let url = "https://github.com/octocat";
        assert!(matches!(parse_github_remote_url(url), Err(Error::NoRemote)));
    }
}
