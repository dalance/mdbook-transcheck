use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_code_comment_header() -> String {
    String::from("# ")
}

fn default_markdown_comment_begin() -> String {
    String::from("(((")
}

fn default_markdown_comment_end() -> String {
    String::from(")))")
}

fn default_similar_threshold() -> f64 {
    0.5
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub excludes: Vec<PathBuf>,
    #[serde(default)]
    pub matcher: ConfigMatcher,
    #[serde(default)]
    pub linter: ConfigLinter,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            excludes: Vec::new(),
            matcher: ConfigMatcher::default(),
            linter: ConfigLinter::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigMatcher {
    #[serde(default)]
    pub enable_code_comment_tweak: bool,
    #[serde(default = "default_code_comment_header")]
    pub code_comment_header: String,
    #[serde(default)]
    pub keep_markdown_comment: bool,
    #[serde(default = "default_markdown_comment_begin")]
    pub markdown_comment_begin: String,
    #[serde(default = "default_markdown_comment_end")]
    pub markdown_comment_end: String,
    #[serde(default = "default_similar_threshold")]
    pub similar_threshold: f64,
}

impl Default for ConfigMatcher {
    fn default() -> Self {
        ConfigMatcher {
            enable_code_comment_tweak: false,
            code_comment_header: String::from("# "),
            keep_markdown_comment: false,
            markdown_comment_begin: String::from("((("),
            markdown_comment_end: String::from(")))"),
            similar_threshold: 0.5,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigLinter {
    #[serde(default)]
    pub enable_emphasis_check: bool,
    #[serde(default)]
    pub enable_half_paren_check: bool,
    #[serde(default)]
    pub enable_full_paren_check: bool,
}

impl Default for ConfigLinter {
    fn default() -> Self {
        ConfigLinter {
            enable_emphasis_check: false,
            enable_half_paren_check: false,
            enable_full_paren_check: false,
        }
    }
}
