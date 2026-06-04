use crate::db::models::{BranchComment, BranchWithComments, IssueComment, IssueWithComments};

pub struct ThreadComment {
    pub author: String,
    pub content: String,
    pub created_at: String,
}

pub struct Thread {
    pub title: String,
    pub description: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
    pub comments: Vec<ThreadComment>,
    pub issue_id: Option<i64>,
    pub branch_id: Option<i64>,
}

impl From<IssueWithComments> for Thread {
    fn from(issue_with_comments: IssueWithComments) -> Self {
        let issue = &issue_with_comments.issue;
        Self {
            title: issue
                .name
                .clone()
                .unwrap_or_else(|| format!("Issue #{}", issue.issue_id)),
            description: issue.description.clone(),
            author: issue.author.clone(),
            created_at: issue.created_at.clone(),
            updated_at: issue.updated_at.clone(),
            comments: issue_with_comments
                .comments
                .into_iter()
                .map(Into::into)
                .collect(),
            issue_id: Some(issue.issue_id),
            branch_id: None,
        }
    }
}

impl From<BranchWithComments> for Thread {
    fn from(branch_with_comments: BranchWithComments) -> Self {
        let branch = &branch_with_comments.branch;
        Self {
            title: branch.name.clone(),
            description: branch.description.clone(),
            author: branch.author.clone(),
            created_at: branch.created_at.clone(),
            updated_at: branch.updated_at.clone(),
            comments: branch_with_comments
                .comments
                .into_iter()
                .map(Into::into)
                .collect(),
            issue_id: None,
            branch_id: Some(branch.branch_id),
        }
    }
}

impl From<IssueComment> for ThreadComment {
    fn from(comment: IssueComment) -> Self {
        Self {
            author: comment.author,
            content: comment.content,
            created_at: comment.created_at,
        }
    }
}

impl From<BranchComment> for ThreadComment {
    fn from(comment: BranchComment) -> Self {
        Self {
            author: comment.author,
            content: comment.content,
            created_at: comment.created_at,
        }
    }
}
