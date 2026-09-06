use super::{Provider, RemoteComment, RemoteIssue};
use crate::error::{Error, Result};
use futures_util::StreamExt;

pub struct GitHubProvider {
    client: octocrab::Octocrab,
    owner: String,
    repo: String,
}

impl GitHubProvider {
    pub async fn new(token: &str, owner: &str, repo: &str) -> crate::error::Result<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote::parse_remote_url;

    fn parse(url: &str) -> (String, String) {
        let target = parse_remote_url(url).unwrap_or_else(|e| {
            panic!("expected GitHub URL {url} to parse, got {e:?}");
        });
        assert_eq!(target.forge, crate::remote::Forge::GitHub);
        target
            .project_path
            .split_once('/')
            .map(|(o, r)| (o.to_string(), r.to_string()))
            .expect("github project_path always has exactly owner/repo")
    }

    #[tokio::test]
    async fn github_provider_builds_without_crypto_provider_panic() {
        crate::remote::ensure_default_crypto_provider();
        let provider = GitHubProvider::new("token", "owner", "repo")
            .await
            .expect("GitHub provider should build once a default crypto provider is installed");
        let _ = provider;
    }

    #[test]
    fn parse_ssh_remote_url() {
        let (owner, repo) = parse("git@github.com:octocat/Hello-World.git");
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_https_remote_url() {
        let (owner, repo) = parse("https://github.com/octocat/Hello-World.git");
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_https_remote_url_without_git_suffix() {
        let (owner, repo) = parse("https://github.com/octocat/Hello-World");
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_ssh_scheme_remote_url() {
        let (owner, repo) = parse("ssh://git@github.com/octocat/Hello-World.git");
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_ssh_scheme_remote_url_without_git_suffix() {
        let (owner, repo) = parse("ssh://git@github.com/octocat/Hello-World");
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_ssh_scheme_remote_url_with_port() {
        let (owner, repo) = parse("ssh://git@github.com:22/octocat/Hello-World.git");
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_ssh_scheme_remote_url_with_port_no_git_suffix() {
        let (owner, repo) = parse("ssh://git@github.com:22/octocat/Hello-World");
        assert_eq!(owner, "octocat");
        assert_eq!(repo, "Hello-World");
    }

    #[test]
    fn parse_gitlab_remote_url_is_not_github() {
        let target = parse_remote_url("https://gitlab.com/octocat/Hello-World.git")
            .expect("gitlab URL should parse");
        assert_eq!(target.forge, crate::remote::Forge::GitLab);
    }

    #[test]
    fn parse_invalid_github_remote_url() {
        let url = "https://github.com/octocat";
        assert!(matches!(
            parse_remote_url(url),
            Err(Error::MalformedRemote(_))
        ));
    }
}
