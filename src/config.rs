use serde::Deserialize;
use std::collections::HashMap;
use toml;
use tree_sitter::Node;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    filetype: HashMap<String, FiletypeConfig>,
}

#[derive(Clone, Deserialize, Default)]
pub struct FiletypeConfig {
    blacklist: Option<Vec<String>>,
    whitelist: Option<Vec<String>>,
    #[serde(default)]
    group: HashMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        let mut config = Config {
            filetype: HashMap::default(),
        };
        config
            .filetype
            .insert("default".to_owned(), FiletypeConfig::default());
        config
    }
}

impl Config {
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Option<Self> {
        let config = std::fs::read_to_string(path).ok()?;
        let mut config: Config = toml::from_str(&config).ok()?;
        if config.filetype.get("default").is_none() {
            config
                .filetype
                .insert("default".to_owned(), FiletypeConfig::default());
        }
        Some(config)
    }

    pub fn get_filetype_config<'a>(&'a self, filetype: &str) -> &'a FiletypeConfig {
        self.filetype
            .get(filetype)
            .or_else(|| self.filetype.get("default"))
            .unwrap()
    }
}

impl FiletypeConfig {
    pub fn is_node_visible(&self, node: Node) -> bool {
        let kind = node.kind();
        match &self.whitelist {
            Some(whitelist) => whitelist.iter().any(|x| x == kind),
            None => match &self.blacklist {
                Some(blacklist) => !blacklist.iter().any(|x| x == kind),
                None => true,
            },
        }
    }

    pub fn resolve_alias<'a>(&'a self, kind: &str) -> Vec<String> {
        self.group
            .get(kind)
            .cloned()
            .unwrap_or_else(|| vec![kind.to_string()])
    }
}
