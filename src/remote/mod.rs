use crate::error::Result;

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

pub mod github;
