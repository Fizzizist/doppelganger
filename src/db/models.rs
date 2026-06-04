#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Author {
    pub author_id: i64,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Issue {
    pub issue_id: i64,
    pub name: String,
    pub description: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Branch {
    pub branch_id: i64,
    pub name: String,
    pub description: String,
    pub author: String,
    pub issue_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct IssueComment {
    pub issue_comment_id: i64,
    pub content: String,
    pub author: String,
    pub issue_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BranchComment {
    pub branch_comment_id: i64,
    pub content: String,
    pub author: String,
    pub branch_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, serde::Serialize)]
pub struct IssueWithComments {
    pub issue: Issue,
    pub comments: Vec<IssueComment>,
}

#[derive(Debug, serde::Serialize)]
pub struct BranchWithComments {
    pub branch: Branch,
    pub comments: Vec<BranchComment>,
}
