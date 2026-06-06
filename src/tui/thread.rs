use crate::db::models::{Branch, BranchComment, Issue, IssueComment};

#[derive(Debug, Clone)]
pub struct Thread {
    pub title: String,
    pub description: String,
    pub author: String,
    pub created_at: String,
    pub comments: Vec<ThreadComment>,
}

#[derive(Debug, Clone)]
pub struct ThreadComment {
    pub author: String,
    pub content: String,
    pub created_at: String,
}

impl From<(&Issue, Vec<IssueComment>)> for Thread {
    fn from((issue, comments): (&Issue, Vec<IssueComment>)) -> Self {
        Thread {
            title: issue
                .name
                .clone()
                .unwrap_or_else(|| format!("Issue #{}", issue.issue_id)),
            description: issue.description.clone(),
            author: issue.author.clone(),
            created_at: issue.created_at.clone(),
            comments: comments.into_iter().map(ThreadComment::from).collect(),
        }
    }
}

impl From<(&Branch, Vec<BranchComment>)> for Thread {
    fn from((branch, comments): (&Branch, Vec<BranchComment>)) -> Self {
        Thread {
            title: branch.name.clone(),
            description: branch.description.clone(),
            author: branch.author.clone(),
            created_at: branch.created_at.clone(),
            comments: comments.into_iter().map(ThreadComment::from).collect(),
        }
    }
}

impl From<IssueComment> for ThreadComment {
    fn from(c: IssueComment) -> Self {
        ThreadComment {
            author: c.author,
            content: c.content,
            created_at: c.created_at,
        }
    }
}

impl From<BranchComment> for ThreadComment {
    fn from(c: BranchComment) -> Self {
        ThreadComment {
            author: c.author,
            content: c.content,
            created_at: c.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_issue() -> Issue {
        Issue {
            issue_id: 1,
            name: Some("Test Issue".to_string()),
            description: "This is a **test** description".to_string(),
            author: "Alice".to_string(),
            created_at: "2025-01-01 00:00:00".to_string(),
            updated_at: "2025-01-01 00:00:00".to_string(),
        }
    }

    fn sample_issue_comment(issue_id: i64, content: &str) -> IssueComment {
        IssueComment {
            issue_comment_id: 1,
            content: content.to_string(),
            author: "Bob".to_string(),
            issue_id,
            created_at: "2025-01-02 00:00:00".to_string(),
            updated_at: "2025-01-02 00:00:00".to_string(),
        }
    }

    fn sample_branch() -> Branch {
        Branch {
            branch_id: 1,
            name: "feature-1".to_string(),
            description: "Branch description".to_string(),
            author: "Carol".to_string(),
            issue_id: 1,
            created_at: "2025-01-01 00:00:00".to_string(),
            updated_at: "2025-01-01 00:00:00".to_string(),
        }
    }

    fn sample_branch_comment(branch_id: i64, content: &str) -> BranchComment {
        BranchComment {
            branch_comment_id: 1,
            content: content.to_string(),
            author: "Dave".to_string(),
            branch_id,
            created_at: "2025-01-03 00:00:00".to_string(),
            updated_at: "2025-01-03 00:00:00".to_string(),
        }
    }

    #[test]
    fn thread_from_issue_with_name() {
        let issue = sample_issue();
        let comments = vec![sample_issue_comment(1, "first comment")];
        let thread = Thread::from((&issue, comments));

        assert_eq!(thread.title, "Test Issue");
        assert_eq!(thread.description, "This is a **test** description");
        assert_eq!(thread.author, "Alice");
        assert_eq!(thread.comments.len(), 1);
        assert_eq!(thread.comments[0].author, "Bob");
        assert_eq!(thread.comments[0].content, "first comment");
    }

    #[test]
    fn thread_from_issue_without_name_uses_issue_number() {
        let mut issue = sample_issue();
        issue.name = None;
        let comments: Vec<IssueComment> = vec![];
        let thread = Thread::from((&issue, comments));

        assert_eq!(thread.title, "Issue #1");
    }

    #[test]
    fn thread_from_issue_with_no_comments() {
        let issue = sample_issue();
        let comments: Vec<IssueComment> = vec![];
        let thread = Thread::from((&issue, comments));

        assert!(thread.comments.is_empty());
    }

    #[test]
    fn thread_from_branch() {
        let branch = sample_branch();
        let comments = vec![sample_branch_comment(1, "branch comment")];
        let thread = Thread::from((&branch, comments));

        assert_eq!(thread.title, "feature-1");
        assert_eq!(thread.description, "Branch description");
        assert_eq!(thread.author, "Carol");
        assert_eq!(thread.comments.len(), 1);
        assert_eq!(thread.comments[0].author, "Dave");
    }

    #[test]
    fn thread_from_branch_with_no_comments() {
        let branch = sample_branch();
        let comments: Vec<BranchComment> = vec![];
        let thread = Thread::from((&branch, comments));

        assert!(thread.comments.is_empty());
    }
}
