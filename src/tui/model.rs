use crate::db::models::{
    Branch, BranchComment, BranchWithComments, Issue, IssueComment, IssueWithComments,
};

pub struct Thread {
    pub title: String,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
    pub comments: Vec<ThreadComment>,
}

pub struct ThreadComment {
    pub author: String,
    pub created_at: String,
    pub content: String,
}

impl From<&Issue> for Thread {
    fn from(issue: &Issue) -> Self {
        let title = issue
            .name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| truncate_to_80(&issue.description));
        Self {
            title,
            author: issue.author.clone(),
            created_at: issue.created_at.clone(),
            updated_at: issue.updated_at.clone(),
            description: issue.description.clone(),
            comments: Vec::new(),
        }
    }
}

impl From<&IssueWithComments> for Thread {
    fn from(issue: &IssueWithComments) -> Self {
        let mut thread: Thread = (&issue.issue).into();
        thread.comments = issue.comments.iter().map(|c| c.into()).collect();
        thread
    }
}

impl From<&Branch> for Thread {
    fn from(branch: &Branch) -> Self {
        Self {
            title: branch.name.clone(),
            author: branch.author.clone(),
            created_at: branch.created_at.clone(),
            updated_at: branch.updated_at.clone(),
            description: branch.description.clone(),
            comments: Vec::new(),
        }
    }
}

impl From<&BranchWithComments> for Thread {
    fn from(branch: &BranchWithComments) -> Self {
        let mut thread: Thread = (&branch.branch).into();
        thread.comments = branch.comments.iter().map(|c| c.into()).collect();
        thread
    }
}

impl From<&IssueComment> for ThreadComment {
    fn from(comment: &IssueComment) -> Self {
        Self {
            author: comment.author.clone(),
            created_at: comment.created_at.clone(),
            content: comment.content.clone(),
        }
    }
}

impl From<&BranchComment> for ThreadComment {
    fn from(comment: &BranchComment) -> Self {
        Self {
            author: comment.author.clone(),
            created_at: comment.created_at.clone(),
            content: comment.content.clone(),
        }
    }
}

fn truncate_to_80(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= 80 {
        s.to_string()
    } else {
        let truncated: String = chars.into_iter().take(80).collect();
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_issue() -> Issue {
        Issue {
            issue_id: 1,
            name: Some("Test Issue".to_string()),
            description: "A test description".to_string(),
            author: "Alice".to_string(),
            created_at: "2025-01-01 00:00:00".to_string(),
            updated_at: "2025-01-02 00:00:00".to_string(),
        }
    }

    fn test_branch() -> Branch {
        Branch {
            branch_id: 1,
            name: "feature-1".to_string(),
            description: "Branch description".to_string(),
            author: "Bob".to_string(),
            issue_id: 1,
            created_at: "2025-01-01 00:00:00".to_string(),
            updated_at: "2025-01-02 00:00:00".to_string(),
        }
    }

    #[test]
    fn thread_from_issue_with_name() {
        let issue = test_issue();
        let thread = Thread::from(&issue);
        assert_eq!(thread.title, "Test Issue");
        assert_eq!(thread.author, "Alice");
        assert!(thread.comments.is_empty());
    }

    #[test]
    fn thread_from_issue_without_name() {
        let mut issue = test_issue();
        issue.name = None;
        issue.description = "x".repeat(100);
        let thread = Thread::from(&issue);
        assert_eq!(thread.title.len(), 83);
        assert!(thread.title.ends_with("..."));
    }

    #[test]
    fn thread_from_issue_short_description_fallback() {
        let mut issue = test_issue();
        issue.name = None;
        issue.description = "Short".to_string();
        let thread = Thread::from(&issue);
        assert_eq!(thread.title, "Short");
    }

    #[test]
    fn thread_from_issue_with_comments() {
        let issue = test_issue();
        let comments = vec![
            IssueComment {
                issue_comment_id: 1,
                content: "First".to_string(),
                author: "Alice".to_string(),
                issue_id: 1,
                created_at: "2025-01-01 00:00:00".to_string(),
                updated_at: "2025-01-01 00:00:00".to_string(),
            },
            IssueComment {
                issue_comment_id: 2,
                content: "Second".to_string(),
                author: "Bob".to_string(),
                issue_id: 1,
                created_at: "2025-01-01 00:00:00".to_string(),
                updated_at: "2025-01-01 00:00:00".to_string(),
            },
        ];
        let iwc = IssueWithComments { issue, comments };
        let thread = Thread::from(&iwc);
        assert_eq!(thread.comments.len(), 2);
        assert_eq!(thread.comments[0].content, "First");
        assert_eq!(thread.comments[1].content, "Second");
    }

    #[test]
    fn thread_from_branch() {
        let branch = test_branch();
        let thread = Thread::from(&branch);
        assert_eq!(thread.title, "feature-1");
        assert_eq!(thread.description, "Branch description");
    }

    #[test]
    fn thread_from_branch_with_comments() {
        let branch = test_branch();
        let comments = vec![
            BranchComment {
                branch_comment_id: 1,
                content: "Note".to_string(),
                author: "Bob".to_string(),
                branch_id: 1,
                created_at: "2025-01-01 00:00:00".to_string(),
                updated_at: "2025-01-01 00:00:00".to_string(),
            },
            BranchComment {
                branch_comment_id: 2,
                content: "Reply".to_string(),
                author: "Alice".to_string(),
                branch_id: 1,
                created_at: "2025-01-01 00:00:00".to_string(),
                updated_at: "2025-01-01 00:00:00".to_string(),
            },
        ];
        let bwc = BranchWithComments { branch, comments };
        let thread = Thread::from(&bwc);
        assert_eq!(thread.comments.len(), 2);
        assert_eq!(thread.comments[0].content, "Note");
        assert_eq!(thread.comments[1].content, "Reply");
    }

    #[test]
    fn thread_comment_from_issue_comment() {
        let ic = IssueComment {
            issue_comment_id: 1,
            content: "Hello".to_string(),
            author: "Alice".to_string(),
            issue_id: 1,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
        };
        let tc = ThreadComment::from(&ic);
        assert_eq!(tc.author, "Alice");
        assert_eq!(tc.content, "Hello");
        assert_eq!(tc.created_at, "2025-01-01");
    }

    #[test]
    fn thread_comment_from_branch_comment() {
        let bc = BranchComment {
            branch_comment_id: 1,
            content: "World".to_string(),
            author: "Bob".to_string(),
            branch_id: 1,
            created_at: "2025-01-01".to_string(),
            updated_at: "2025-01-01".to_string(),
        };
        let tc = ThreadComment::from(&bc);
        assert_eq!(tc.author, "Bob");
        assert_eq!(tc.content, "World");
    }
}
