use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::{Error, Result};

fn default_editor() -> String {
    "nvim".to_string()
}

const SAMPLE_CONFIG: &str = r#"# doppelganger configuration
#
# default_human_author: used when --human or --author default_human_author is passed
# default_robot_author: used by default (robot/automated commits)
# profiles.<id>: additional named author profiles
# editor: text editor command for TUI content editing (default: nvim)

[default_human_author]
name = "Your Name"
email = "you@example.com"

[default_robot_author]
name = "doppelganger"
# email is optional for robot profiles

# [profiles.ci]
# name = "CI Bot"
# email = "ci@example.com"

# [github]
# token = "ghp_xxxxxxxxxxxx"

# [gitlab]
# token = "glpat-xxxxxxxxxxxx"
"#;

#[derive(Debug, Deserialize, Clone)]
pub struct AuthorProfile {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubConfig {
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitLabConfig {
    pub token: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub default_human_author: Option<AuthorProfile>,
    pub default_robot_author: Option<AuthorProfile>,
    #[serde(default)]
    pub profiles: HashMap<String, AuthorProfile>,
    #[serde(default = "default_editor")]
    pub editor: String,
    pub github: Option<GitHubConfig>,
    pub gitlab: Option<GitLabConfig>,
}

#[derive(Debug, Clone)]
pub enum AuthorSelection {
    Robot,
    Human,
    Named(String),
}

impl Config {
    pub fn resolve(&self, selection: AuthorSelection) -> Result<(String, Option<String>)> {
        let profile = match selection {
            AuthorSelection::Robot => self
                .default_robot_author
                .as_ref()
                .ok_or_else(|| Error::MissingProfileField("default_robot_author".to_string()))?,
            AuthorSelection::Human => self
                .default_human_author
                .as_ref()
                .ok_or_else(|| Error::MissingProfileField("default_human_author".to_string()))?,
            AuthorSelection::Named(ref id) if id == "default_human_author" => self
                .default_human_author
                .as_ref()
                .ok_or_else(|| Error::MissingProfileField("default_human_author".to_string()))?,
            AuthorSelection::Named(ref id) if id == "default_robot_author" => self
                .default_robot_author
                .as_ref()
                .ok_or_else(|| Error::MissingProfileField("default_robot_author".to_string()))?,
            AuthorSelection::Named(ref id) => self
                .profiles
                .get(id)
                .ok_or_else(|| Error::UnknownProfile(id.clone()))?,
        };
        Ok((profile.name.clone(), profile.email.clone()))
    }
}

#[derive(Debug)]
pub enum LoadOutcome {
    Created(PathBuf, Config),
    Loaded(Config),
}

pub fn config_path() -> Result<PathBuf> {
    let base = if let Ok(override_dir) = std::env::var("DOPPELGANGER_CONFIG_DIR") {
        PathBuf::from(override_dir)
    } else {
        dirs::config_dir().ok_or(Error::ConfigDirUnavailable)?
    };
    Ok(base.join("doppelganger").join("config.toml"))
}

pub fn load_or_init() -> Result<LoadOutcome> {
    load_from_path(config_path()?)
}

fn load_from_path(path: PathBuf) -> Result<LoadOutcome> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => {
            use std::io::Write;
            write!(&file, "{}", SAMPLE_CONFIG)?;
            drop(file);
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            return Ok(LoadOutcome::Created(path, config));
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e.into()),
    }

    let content = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;

    for reserved in ["default_human_author", "default_robot_author"] {
        if config.profiles.contains_key(reserved) {
            return Err(Error::DuplicateProfile(reserved.to_string()));
        }
    }

    Ok(LoadOutcome::Loaded(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_config_dir() -> TempDir {
        tempfile::tempdir().expect("temp dir")
    }

    fn write_config(dir: &TempDir, toml: &str) -> PathBuf {
        let cfg_dir = dir.path().join("doppelganger");
        std::fs::create_dir_all(&cfg_dir).expect("create config dir");
        let path = cfg_dir.join("config.toml");
        std::fs::write(&path, toml).expect("write config");
        path
    }

    fn config_file_path(dir: &TempDir) -> PathBuf {
        dir.path().join("doppelganger").join("config.toml")
    }

    #[test]
    fn config_path_honors_env_override() {
        let dir = temp_config_dir();
        // SAFETY: single-threaded test; no concurrent env reads in this binary
        unsafe {
            std::env::set_var("DOPPELGANGER_CONFIG_DIR", dir.path());
        }
        let path = config_path().expect("config_path should succeed");
        assert_eq!(path, dir.path().join("doppelganger").join("config.toml"));
        unsafe {
            std::env::remove_var("DOPPELGANGER_CONFIG_DIR");
        }
    }

    #[test]
    fn load_or_init_creates_sample_on_first_run() {
        let dir = temp_config_dir();
        let path = config_file_path(&dir);
        let outcome = load_from_path(path).expect("load_from_path should succeed");
        match outcome {
            LoadOutcome::Created(p, _config) => {
                assert!(p.exists(), "sample file should be written");
                let content = std::fs::read_to_string(&p).expect("read sample");
                assert!(content.contains("default_human_author"));
                assert!(content.contains("default_robot_author"));
            }
            LoadOutcome::Loaded(_) => panic!("expected Created, got Loaded"),
        }
    }

    #[test]
    fn parse_valid_config() {
        let dir = temp_config_dir();
        let path = write_config(
            &dir,
            r#"
[default_human_author]
name = "Alice"
email = "alice@example.com"

[default_robot_author]
name = "Bot"

[profiles.ci]
name = "CI Bot"
email = "ci@example.com"
"#,
        );
        let outcome = load_from_path(path).expect("should load");
        match outcome {
            LoadOutcome::Loaded(config) => {
                assert_eq!(
                    config.default_human_author.as_ref().map(|p| &p.name),
                    Some(&"Alice".to_string())
                );
                assert_eq!(
                    config
                        .default_robot_author
                        .as_ref()
                        .map(|p| p.email.as_ref()),
                    Some(None)
                );
                assert!(config.profiles.contains_key("ci"));
            }
            LoadOutcome::Created(_, _) => panic!("expected Loaded, got Created"),
        }
    }

    #[test]
    fn resolve_robot_default() {
        let config = Config {
            default_robot_author: Some(AuthorProfile {
                name: "Bot".to_string(),
                email: None,
            }),
            default_human_author: Some(AuthorProfile {
                name: "Human".to_string(),
                email: Some("h@example.com".to_string()),
            }),
            profiles: HashMap::new(),
            editor: default_editor(),
            github: None,
            gitlab: None,
        };
        let (name, email) = config.resolve(AuthorSelection::Robot).expect("resolve");
        assert_eq!(name, "Bot");
        assert_eq!(email, None);
    }

    #[test]
    fn resolve_human() {
        let config = Config {
            default_human_author: Some(AuthorProfile {
                name: "Alice".to_string(),
                email: Some("alice@example.com".to_string()),
            }),
            default_robot_author: None,
            profiles: HashMap::new(),
            editor: default_editor(),
            github: None,
            gitlab: None,
        };
        let (name, email) = config.resolve(AuthorSelection::Human).expect("resolve");
        assert_eq!(name, "Alice");
        assert_eq!(email.as_deref(), Some("alice@example.com"));
    }

    #[test]
    fn resolve_named_default_human_author_equivalent_to_human() {
        let config = Config {
            default_human_author: Some(AuthorProfile {
                name: "Alice".to_string(),
                email: Some("alice@example.com".to_string()),
            }),
            default_robot_author: None,
            profiles: HashMap::new(),
            editor: default_editor(),
            github: None,
            gitlab: None,
        };
        let (name1, _) = config
            .resolve(AuthorSelection::Human)
            .expect("resolve human");
        let (name2, _) = config
            .resolve(AuthorSelection::Named("default_human_author".to_string()))
            .expect("resolve named");
        assert_eq!(name1, name2);
    }

    #[test]
    fn resolve_named_extra_profile() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "ci".to_string(),
            AuthorProfile {
                name: "CI Bot".to_string(),
                email: Some("ci@example.com".to_string()),
            },
        );
        let config = Config {
            default_human_author: None,
            default_robot_author: None,
            profiles,
            editor: default_editor(),
            github: None,
            gitlab: None,
        };
        let (name, email) = config
            .resolve(AuthorSelection::Named("ci".to_string()))
            .expect("resolve");
        assert_eq!(name, "CI Bot");
        assert_eq!(email.as_deref(), Some("ci@example.com"));
    }

    #[test]
    fn resolve_unknown_profile_errors() {
        let config = Config::default();
        let result = config.resolve(AuthorSelection::Named("ghost".to_string()));
        match result {
            Err(Error::UnknownProfile(id)) => assert_eq!(id, "ghost"),
            other => panic!("expected UnknownProfile, got {other:?}"),
        }
    }

    #[test]
    fn resolve_missing_required_robot_errors() {
        let config = Config::default();
        let result = config.resolve(AuthorSelection::Robot);
        match result {
            Err(Error::MissingProfileField(f)) => assert_eq!(f, "default_robot_author"),
            other => panic!("expected MissingProfileField, got {other:?}"),
        }
    }

    #[test]
    fn resolve_missing_required_human_errors() {
        let config = Config::default();
        let result = config.resolve(AuthorSelection::Human);
        match result {
            Err(Error::MissingProfileField(f)) => assert_eq!(f, "default_human_author"),
            other => panic!("expected MissingProfileField, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_profile_identifier_errors() {
        let dir = temp_config_dir();
        let path = write_config(
            &dir,
            r#"
[default_human_author]
name = "Alice"
email = "alice@example.com"

[default_robot_author]
name = "Bot"

[profiles.default_human_author]
name = "Sneaky"
"#,
        );
        let result = load_from_path(path);
        match result {
            Err(Error::DuplicateProfile(id)) => assert_eq!(id, "default_human_author"),
            other => panic!("expected DuplicateProfile, got {other:?}"),
        }
    }

    #[test]
    fn malformed_toml_yields_config_parse_error() {
        let dir = temp_config_dir();
        let path = write_config(&dir, "this is [not valid {{{ toml");
        let result = load_from_path(path);
        match result {
            Err(Error::ConfigParse(_)) => {}
            other => panic!("expected ConfigParse, got {other:?}"),
        }
    }

    #[test]
    fn robot_without_email_yields_none() {
        let config = Config {
            default_robot_author: Some(AuthorProfile {
                name: "Bot".to_string(),
                email: None,
            }),
            default_human_author: None,
            profiles: HashMap::new(),
            editor: default_editor(),
            github: None,
            gitlab: None,
        };
        let (_, email) = config.resolve(AuthorSelection::Robot).expect("resolve");
        assert_eq!(email, None);
    }

    #[test]
    fn config_with_github_token() {
        let toml = r#"
[default_human_author]
name = "Alice"
email = "alice@example.com"

[default_robot_author]
name = "Bot"

[github]
token = "ghp_test123"
"#;
        let config: Config = toml::from_str(toml).expect("parse");
        assert_eq!(
            config.github.as_ref().map(|g| g.token.as_str()),
            Some("ghp_test123")
        );
    }

    #[test]
    fn config_without_github_section() {
        let toml = r#"
[default_human_author]
name = "Alice"
email = "alice@example.com"

[default_robot_author]
name = "Bot"
"#;
        let config: Config = toml::from_str(toml).expect("parse");
        assert!(config.github.is_none());
    }

    #[test]
    fn config_with_gitlab_token() {
        let toml = r#"
[default_human_author]
name = "Alice"
email = "alice@example.com"

[default_robot_author]
name = "Bot"

[gitlab]
token = "glpat_test123"
"#;
        let config: Config = toml::from_str(toml).expect("parse");
        assert_eq!(
            config.gitlab.as_ref().map(|g| g.token.as_str()),
            Some("glpat_test123")
        );
    }

    #[test]
    fn config_without_gitlab_section() {
        let toml = r#"
[default_human_author]
name = "Alice"
email = "alice@example.com"

[default_robot_author]
name = "Bot"
"#;
        let config: Config = toml::from_str(toml).expect("parse");
        assert!(config.gitlab.is_none());
    }

    #[test]
    fn config_without_editor_defaults_to_nvim() {
        let toml = r#"
[default_human_author]
name = "Alice"
email = "alice@example.com"
"#;
        let config: Config = toml::from_str(toml).expect("parse");
        assert_eq!(config.editor, "nvim");
    }

    #[test]
    fn config_with_explicit_editor() {
        let toml = r#"
editor = "nano"

[default_human_author]
name = "Alice"
email = "alice@example.com"
"#;
        let config: Config = toml::from_str(toml).expect("parse");
        assert_eq!(config.editor, "nano");
    }
}
