pub const AUTHOR_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS author (
    author_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT
)"#;

pub const ISSUE_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS issue (
    issue_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,
    description TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES author(author_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)"#;

pub const BRANCH_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS branch (
    branch_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES author(author_id),
    issue_id INTEGER NOT NULL REFERENCES issue(issue_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)"#;

pub const ISSUE_COMMENT_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS issue_comment (
    issue_comment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES author(author_id),
    issue_id INTEGER NOT NULL REFERENCES issue(issue_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)"#;

pub const BRANCH_COMMENT_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS branch_comment (
    branch_comment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES author(author_id),
    branch_id INTEGER NOT NULL REFERENCES branch(branch_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)"#;
