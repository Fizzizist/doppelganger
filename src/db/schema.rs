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
    name TEXT NOT NULL,
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

pub const ALTER_TABLE_ISSUE_REMOTE_ID: &str = "ALTER TABLE issue ADD COLUMN remote_id TEXT";
pub const ALTER_TABLE_ISSUE_ARCHIVED_AT: &str = "ALTER TABLE issue ADD COLUMN archived_at TEXT";
pub const ALTER_TABLE_ISSUE_COMMENT_HIDDEN_AT: &str =
    "ALTER TABLE issue_comment ADD COLUMN hidden_at TEXT";
pub const ALTER_TABLE_BRANCH_COMMENT_HIDDEN_AT: &str =
    "ALTER TABLE branch_comment ADD COLUMN hidden_at TEXT";

pub const ALTER_TABLE_BRANCH_ARCHIVED_AT: &str = "ALTER TABLE branch ADD COLUMN archived_at TEXT";

pub const CREATE_INDEX_BRANCH_ACTIVE_NAME: &str = "CREATE UNIQUE INDEX IF NOT EXISTS idx_branch_active_name ON branch(name) WHERE archived_at IS NULL";

pub const BRANCH_TABLE_NEW: &str = r#"
CREATE TABLE IF NOT EXISTS branch_new (
    branch_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES author(author_id),
    issue_id INTEGER NOT NULL REFERENCES issue(issue_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    archived_at TEXT
)"#;

pub const DROP_BRANCH_OLD: &str = "DROP TABLE IF EXISTS branch";

pub const RENAME_BRANCH_NEW: &str = "ALTER TABLE branch_new RENAME TO branch";
