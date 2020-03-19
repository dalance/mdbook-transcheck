use serde_derive::{Deserialize, Serialize};

fn default_code_comment_header() -> String {
    String::from("# ")
}

fn default_similar_threshold() -> f64 {
    0.5
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub enable_code_comment_tweak: bool,
    #[serde(default = "default_code_comment_header")]
    pub code_comment_header: String,
    #[serde(default = "default_similar_threshold")]
    pub similar_threshold: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enable_code_comment_tweak: false,
            code_comment_header: String::from("# "),
            similar_threshold: 0.5,
        }
    }
}
