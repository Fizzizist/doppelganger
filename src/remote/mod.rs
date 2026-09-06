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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forge {
    GitHub,
    GitLab,
}

#[derive(Debug)]
pub struct RemoteTarget {
    pub forge: Forge,
    pub host: String,
    pub project_path: String,
    pub secure: bool,
}

pub fn ensure_default_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

pub async fn provider_from_config(
    config: &crate::config::Config,
    repo: &git2::Repository,
) -> Result<Box<dyn Provider>> {
    let url = crate::git::remote_url(repo)?;
    let target = parse_remote_url(&url)?;
    let missing_token = |forge: &str| {
        Error::RemoteSync(format!(
            "remote token not configured; add a [{forge}] section with token to your config"
        ))
    };
    match target.forge {
        Forge::GitHub => {
            let github_config = config
                .github
                .as_ref()
                .ok_or_else(|| missing_token("github"))?;
            let (owner, repo_name) = target
                .project_path
                .split_once('/')
                .expect("github project_path always has exactly owner/repo");
            let provider =
                github::GitHubProvider::new(&github_config.token, owner, repo_name).await?;
            Ok(Box::new(provider))
        }
        Forge::GitLab => {
            let gitlab_config = config
                .gitlab
                .as_ref()
                .ok_or_else(|| missing_token("gitlab"))?;
            let provider = gitlab::GitLabProvider::new(
                &gitlab_config.token,
                &target.project_path,
                &target.host,
                target.secure,
            )
            .await?;
            Ok(Box::new(provider))
        }
    }
}

pub fn parse_remote_url(url: &str) -> Result<RemoteTarget> {
    let (host, path, secure) =
        split_remote_url(url).ok_or_else(|| Error::MalformedRemote(url.to_string()))?;
    let forge = match host.to_ascii_lowercase().as_str() {
        "github.com" => Forge::GitHub,
        "gitlab.com" => Forge::GitLab,
        _ => return Err(Error::UnsupportedHost(host.clone())),
    };
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let valid = match forge {
        Forge::GitHub => segments.len() == 2,
        Forge::GitLab => segments.len() >= 2,
    };
    if !valid {
        return Err(Error::MalformedRemote(url.to_string()));
    }
    Ok(RemoteTarget {
        forge,
        host,
        project_path: segments.join("/"),
        secure,
    })
}

fn split_remote_url(url: &str) -> Option<(String, String, bool)> {
    let url = url.trim();
    let url = url.strip_suffix(".git").unwrap_or(url);

    for (scheme, secure) in [("https://", true), ("http://", false), ("ssh://", true)] {
        if let Some(rest) = url.strip_prefix(scheme) {
            let rest = if scheme == "ssh://" {
                rest.strip_prefix("git@").unwrap_or(rest)
            } else {
                rest
            };
            let (host, path) = split_authority_path(rest)?;
            return Some((host, path, secure));
        }
    }

    let (host_part, path) = url.split_once(':')?;
    let host = host_part.rsplit('@').next()?;
    if host.is_empty() || path.is_empty() {
        return None;
    }
    Some((host.to_string(), path.to_string(), true))
}

fn split_authority_path(rest: &str) -> Option<(String, String)> {
    let (authority, path) = rest.split_once('/')?;
    let authority = authority.rsplit('@').next()?;
    let host = authority.split(':').next()?;
    if host.is_empty() || path.is_empty() {
        return None;
    }
    Some((host.to_string(), path.to_string()))
}

pub mod github;
pub mod gitlab;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_github_from_scp_style_url() {
        let target = parse_remote_url("git@github.com:octocat/Hello-World.git").unwrap();
        assert_eq!(target.forge, Forge::GitHub);
    }

    #[test]
    fn detect_github_from_https_url() {
        let target = parse_remote_url("https://github.com/octocat/Hello-World.git").unwrap();
        assert_eq!(target.forge, Forge::GitHub);
    }

    #[test]
    fn detect_github_from_ssh_scheme_url() {
        let target = parse_remote_url("ssh://git@github.com/octocat/Hello-World").unwrap();
        assert_eq!(target.forge, Forge::GitHub);
    }

    #[test]
    fn detect_github_from_ssh_scheme_url_with_port() {
        let target = parse_remote_url("ssh://git@github.com:22/octocat/Hello-World.git").unwrap();
        assert_eq!(target.forge, Forge::GitHub);
    }

    #[test]
    fn detect_gitlab_from_scp_style_url() {
        let target = parse_remote_url("git@gitlab.com:group/subgroup/project.git").unwrap();
        assert_eq!(target.forge, Forge::GitLab);
    }

    #[test]
    fn detect_gitlab_from_https_url() {
        let target = parse_remote_url("https://gitlab.com/group/project.git").unwrap();
        assert_eq!(target.forge, Forge::GitLab);
    }

    #[test]
    fn detect_gitlab_from_ssh_scheme_url_with_port() {
        let target = parse_remote_url("ssh://git@gitlab.com:2222/group/project").unwrap();
        assert_eq!(target.forge, Forge::GitLab);
    }

    #[test]
    fn parse_github_target_owner_repo() {
        let target = parse_remote_url("git@github.com:octocat/Hello-World.git").unwrap();
        assert_eq!(target.project_path, "octocat/Hello-World");
        assert_eq!(target.host, "github.com");
        assert!(target.secure);
    }

    #[test]
    fn parse_gitlab_target_nested_groups() {
        let target = parse_remote_url("https://gitlab.com/group/subgroup/project.git").unwrap();
        assert_eq!(target.project_path, "group/subgroup/project");
        assert_eq!(target.host, "gitlab.com");
        assert!(target.secure);
    }

    #[test]
    fn parse_http_target_is_not_secure() {
        let target = parse_remote_url("http://gitlab.com/group/project").unwrap();
        assert!(!target.secure);
        assert_eq!(target.forge, Forge::GitLab);
    }

    #[test]
    fn parse_unsupported_host_errors() {
        let url = "https://bitbucket.org/owner/repo.git";
        match parse_remote_url(url) {
            Err(Error::UnsupportedHost(remote)) => assert_eq!(remote, "bitbucket.org"),
            other => panic!("expected UnsupportedHost, got {other:?}"),
        }
    }

    #[test]
    fn parse_github_path_with_one_segment_errors() {
        let url = "https://github.com/octocat";
        assert!(matches!(
            parse_remote_url(url),
            Err(Error::MalformedRemote(_))
        ));
    }

    #[test]
    fn parse_gitlab_path_with_one_segment_errors() {
        let url = "https://gitlab.com/project";
        assert!(matches!(
            parse_remote_url(url),
            Err(Error::MalformedRemote(_))
        ));
    }
}
