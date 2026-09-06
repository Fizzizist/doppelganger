use super::{Provider, RemoteComment, RemoteIssue};
use crate::error::{Error, Result};
use gitlab::{
    AsyncGitlab,
    api::{self, AsyncQuery, Pagination},
};
use serde::Deserialize;

const CONSTRUCTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub struct GitLabProvider {
    client: AsyncGitlab,
    project_path: String,
    host: String,
}

#[derive(Debug, Deserialize)]
struct GitLabIssue {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    author: Option<GitLabUser>,
}

#[derive(Debug, Deserialize)]
struct GitLabNote {
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    author: Option<GitLabUser>,
    #[serde(default)]
    system: bool,
}

#[derive(Debug, Default, Deserialize)]
struct GitLabUser {
    #[serde(default)]
    username: Option<String>,
}

fn error_chain(e: &dyn std::error::Error) -> String {
    let mut chain = String::new();
    let mut source = e.source();
    while let Some(cause) = source {
        chain.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    chain
}

impl GitLabProvider {
    pub async fn new(token: &str, project_path: &str, host: &str, secure: bool) -> Result<Self> {
        let mut builder = gitlab::GitlabBuilder::new(host, token);
        if !secure {
            builder.insecure();
        }
        let build = tokio::time::timeout(CONSTRUCTION_TIMEOUT, builder.build_async()).await;
        let client = match build {
            Ok(Ok(client)) => client,
            Ok(Err(e)) => {
                return Err(Error::Remote(format!(
                    "failed to build GitLab client: {e}{}",
                    error_chain(&e)
                )));
            }
            Err(_) => {
                return Err(Error::Remote(format!(
                    "timed out building GitLab client after {}s",
                    CONSTRUCTION_TIMEOUT.as_secs()
                )));
            }
        };
        Ok(Self {
            client,
            project_path: project_path.to_string(),
            host: host.to_string(),
        })
    }
}

#[async_trait::async_trait]
impl Provider for GitLabProvider {
    async fn fetch_issue(&self, issue_iid: i64) -> Result<RemoteIssue> {
        if issue_iid <= 0 {
            return Err(Error::Validation(
                "issue number must be a positive integer".to_string(),
            ));
        }
        let iid = issue_iid as u64;

        let endpoint = gitlab::api::projects::issues::Issue::builder()
            .project(self.project_path.as_str())
            .issue(iid)
            .build()
            .map_err(|e| Error::Remote(format!("failed to build GitLab issue query: {e}")))?;
        let issue: GitLabIssue = endpoint.query_async(&self.client).await.map_err(|e| {
            Error::Remote(format!(
                "failed to fetch GitLab issue !{issue_iid} from {}: {e}",
                self.project_path
            ))
        })?;

        let notes_endpoint = gitlab::api::projects::issues::notes::IssueNotes::builder()
            .project(self.project_path.as_str())
            .issue(iid)
            .order_by(gitlab::api::projects::issues::notes::NoteOrderBy::CreatedAt)
            .sort(gitlab::api::common::SortOrder::Ascending)
            .build()
            .map_err(|e| Error::Remote(format!("failed to build GitLab notes query: {e}")))?;
        let notes: Vec<GitLabNote> = api::paged(notes_endpoint, Pagination::All)
            .query_async(&self.client)
            .await
            .map_err(|e| {
                Error::Remote(format!(
                    "failed to fetch notes for GitLab issue !{issue_iid} from {}: {e}",
                    self.project_path
                ))
            })?;

        let comments = notes
            .into_iter()
            .filter(|note| !note.system)
            .map(|note| {
                let author = note.author.unwrap_or_default().username.unwrap_or_default();
                let body = note.body.unwrap_or_default();
                RemoteComment { author, body }
            })
            .collect();

        let remote_id = format!("gl:{}/{}#{}", self.host, self.project_path, issue_iid);

        Ok(RemoteIssue {
            remote_id,
            title: issue.title,
            body: issue.description.unwrap_or_default(),
            author: issue
                .author
                .unwrap_or_default()
                .username
                .unwrap_or_default(),
            comments,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const ISSUE_BODY: &str = r#"{
        "id": 501,
        "iid": 1,
        "title": "Refactor the warp core",
        "description": "The dilithium matrix is unstable.",
        "state": "opened",
        "author": {"id": 1, "username": "spock"}
    }"#;

    fn note(body: &str, username: &str, system: bool) -> serde_json::Value {
        json!({
            "body": body,
            "system": system,
            "author": {"id": 2, "username": username}
        })
    }

    fn first_page_notes() -> Vec<serde_json::Value> {
        let mut notes = vec![
            note("first comment", "kirk", false),
            note("changed the description", "gitlab-bot", true),
        ];
        notes.extend((2..100).map(|i| note(&format!("filler {i}"), &format!("crew{i}"), false)));
        notes
    }

    async fn mount_user_endpoint(server: &MockServer) {
        Mock::given(method("GET"))
            .and(path("/api/v4/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 1, "username": "test"
            })))
            .mount(server)
            .await;
    }

    async fn mock_gitlab() -> MockServer {
        let server = MockServer::start().await;

        mount_user_endpoint(&server).await;

        Mock::given(method("GET"))
            .and(path("/api/v4/projects/testowner%2Ftestrepo/issues/1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ISSUE_BODY))
            .mount(&server)
            .await;

        let notes_path = "/api/v4/projects/testowner%2Ftestrepo/issues/1/notes";

        Mock::given(method("GET"))
            .and(path(notes_path))
            .and(query_param("page", "1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(first_page_notes())
                    .insert_header(
                        "link",
                        format!(
                            "<{}/api/v4/projects/testowner%2Ftestrepo/issues/1/notes?page=2&per_page=100>; rel=\"next\"",
                            server.uri()
                        ),
                    ),
            )
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path(notes_path))
            .and(query_param("page", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![note(
                "second comment",
                "mccoy",
                false,
            )]))
            .mount(&server)
            .await;

        server
    }

    async fn provider_for(server: &MockServer) -> GitLabProvider {
        let host = server
            .uri()
            .strip_prefix("http://")
            .expect("mock server uri")
            .to_string();
        GitLabProvider::new("glpat_token", "testowner/testrepo", &host, false)
            .await
            .expect("build GitLab provider against mock server")
    }

    #[tokio::test]
    async fn fetch_issue_returns_issue_with_chronological_notes() {
        let server = mock_gitlab().await;
        let provider = provider_for(&server).await;

        let issue = provider.fetch_issue(1).await.expect("fetch issue");

        let uri = server.uri();
        let host = uri.strip_prefix("http://").expect("mock server uri");
        assert_eq!(issue.remote_id, format!("gl:{host}/testowner/testrepo#1"));
        assert_eq!(issue.title.as_deref(), Some("Refactor the warp core"));
        assert_eq!(issue.body, "The dilithium matrix is unstable.");
        assert_eq!(issue.author, "spock");
        assert_eq!(issue.comments.len(), 100);
        assert_eq!(issue.comments[0].author, "kirk");
        assert_eq!(issue.comments[0].body, "first comment");
        assert_eq!(issue.comments[1].body, "filler 2");
        assert_eq!(issue.comments[99].author, "mccoy");
        assert_eq!(issue.comments[99].body, "second comment");
    }

    #[tokio::test]
    async fn fetch_issue_404_errors_mentioning_gitlab() {
        let server = MockServer::start().await;

        mount_user_endpoint(&server).await;

        Mock::given(method("GET"))
            .and(path("/api/v4/projects/testowner%2Ftestrepo/issues/99"))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(json!({"message": "404 Issue Not Found"})),
            )
            .mount(&server)
            .await;

        let provider = provider_for(&server).await;
        match provider.fetch_issue(99).await {
            Err(Error::Remote(msg)) => {
                assert!(msg.to_lowercase().contains("gitlab"), "got: {msg}");
            }
            Ok(other) => panic!(
                "expected Error::Remote, got Ok with title {:?}",
                other.title
            ),
            Err(other) => panic!("expected Error::Remote, got {other}"),
        }
    }

    #[tokio::test]
    async fn fetch_issue_server_error_errors() {
        let server = MockServer::start().await;

        mount_user_endpoint(&server).await;

        Mock::given(method("GET"))
            .and(path("/api/v4/projects/testowner%2Ftestrepo/issues/1"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let provider = provider_for(&server).await;
        assert!(matches!(
            provider.fetch_issue(1).await,
            Err(Error::Remote(_))
        ));
    }

    #[tokio::test]
    async fn fetch_issue_non_positive_iid_errors() {
        let server = mock_gitlab().await;
        let provider = provider_for(&server).await;
        let result = provider.fetch_issue(0).await;
        assert!(matches!(result, Err(Error::Validation(_))));
    }

    #[test]
    fn issue_dto_tolerates_null_fields() {
        let body = r#"{
            "id": 1, "iid": 2, "title": null, "description": null,
            "state": "opened", "author": {"id": 1, "username": null}
        }"#;
        let issue: GitLabIssue = serde_json::from_str(body).expect("deserialize");
        assert!(issue.title.is_none());
        assert!(issue.description.is_none());
        assert!(issue.author.is_some());
        assert_eq!(issue.author.expect("author present").username, None);
    }

    #[test]
    fn issue_dto_tolerates_null_author() {
        let body = r#"{"id": 1, "iid": 2, "state": "opened", "author": null}"#;
        let issue: GitLabIssue = serde_json::from_str(body).expect("deserialize");
        assert!(issue.author.is_none());
    }

    #[test]
    fn note_dto_defaults_missing_system_flag() {
        let body = r#"{"body": "hello", "author": {"id": 1, "username": "uhura"}}"#;
        let note: GitLabNote = serde_json::from_str(body).expect("deserialize");
        assert!(!note.system);
        assert_eq!(note.body.as_deref(), Some("hello"));
    }

    #[test]
    fn note_dto_tolerates_null_author() {
        let body = r#"{"body": "hello", "author": null, "system": false}"#;
        let note: GitLabNote = serde_json::from_str(body).expect("deserialize");
        assert!(note.author.is_none());
    }
}
