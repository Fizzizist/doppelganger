use crate::db::models::{Branch, BranchComment, Issue, IssueComment};

pub struct Thread {
    pub thread_id: i64,
    pub title: String,
    pub description: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct ThreadComment {
    pub comment_id: i64,
    pub thread_id: i64,
    pub content: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Issue> for Thread {
    fn from(value: Issue) -> Self {
        Self {
            thread_id: value.issue_id,
            title: value.name.unwrap_or_else(|| format!("#{}", value.issue_id)),
            description: value.description,
            author: value.author,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Branch> for Thread {
    fn from(value: Branch) -> Self {
        Self {
            thread_id: value.branch_id,
            title: value.name,
            description: value.description,
            author: value.author,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<IssueComment> for ThreadComment {
    fn from(value: IssueComment) -> Self {
        Self {
            comment_id: value.issue_comment_id,
            thread_id: value.issue_id,
            content: value.content,
            author: value.author,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<BranchComment> for ThreadComment {
    fn from(value: BranchComment) -> Self {
        Self {
            comment_id: value.branch_comment_id,
            thread_id: value.branch_id,
            content: value.content,
            author: value.author,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

pub mod app;
pub mod event;
pub mod issue_list;
pub mod markdown;
pub mod thread_view;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{Branch, BranchComment, Issue, IssueComment};

    #[test]
    fn thread_from_issue_with_name() {
        let issue = Issue {
            issue_id: 42,
            name: Some("My Issue".to_string()),
            description: "desc".to_string(),
            author: "Alice".to_string(),
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-02".to_string(),
        };
        let thread: Thread = issue.into();
        assert_eq!(thread.thread_id, 42);
        assert_eq!(thread.title, "My Issue");
        assert_eq!(thread.description, "desc");
    }

    #[test]
    fn thread_from_issue_without_name() {
        let issue = Issue {
            issue_id: 7,
            name: None,
            description: "desc".to_string(),
            author: "Bob".to_string(),
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-02".to_string(),
        };
        let thread: Thread = issue.into();
        assert_eq!(thread.title, "#7");
    }

    #[test]
    fn thread_from_branch() {
        let branch = Branch {
            branch_id: 3,
            name: "feature-x".to_string(),
            description: "branch desc".to_string(),
            author: "Carol".to_string(),
            issue_id: 1,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-02".to_string(),
        };
        let thread: Thread = branch.into();
        assert_eq!(thread.thread_id, 3);
        assert_eq!(thread.title, "feature-x");
    }

    #[test]
    fn thread_comment_from_issue_comment() {
        let ic = IssueComment {
            issue_comment_id: 10,
            content: "hello".to_string(),
            author: "Dave".to_string(),
            issue_id: 5,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
        };
        let tc: ThreadComment = ic.into();
        assert_eq!(tc.comment_id, 10);
        assert_eq!(tc.thread_id, 5);
        assert_eq!(tc.content, "hello");
    }

    #[test]
    fn thread_comment_from_branch_comment() {
        let bc = BranchComment {
            branch_comment_id: 20,
            content: "world".to_string(),
            author: "Eve".to_string(),
            branch_id: 8,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
        };
        let tc: ThreadComment = bc.into();
        assert_eq!(tc.comment_id, 20);
        assert_eq!(tc.thread_id, 8);
        assert_eq!(tc.content, "world");
    }
}
