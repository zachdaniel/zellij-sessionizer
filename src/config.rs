use std::collections::BTreeMap;
use std::path::PathBuf;

use zellij_tile::prelude::LayoutInfo;

use crate::ROOT;

#[derive(Debug)]
pub struct Config {
    pub root_dirs: Vec<PathBuf>,  // Directories to search in
    pub dirs: Vec<PathBuf>,       // Specific directories to include directly
    pub layout: LayoutInfo,
    pub root_files: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root_dirs: vec![PathBuf::from(ROOT)],
            dirs: vec![],
            layout: LayoutInfo::BuiltIn("default".to_string()),
            root_files: vec![".git".to_string()]
        }
    }
}

fn parse_layout(layout: &str) -> LayoutInfo {
    // builtin: ":default" custom: "default"
    if layout.starts_with(":") {
        LayoutInfo::BuiltIn(layout.trim_start_matches(':').to_string())
    } else {
        LayoutInfo::File(layout.to_string())
    }
}


fn parse_dirs(dirs: &str) -> Vec<PathBuf> {
    return dirs.split(';').map(PathBuf::from).collect()
}

fn parse_root_files(root_files: &str) -> Vec<String> {
    return root_files.split(';').map(|s| s.to_string()).collect()
}

impl From<BTreeMap<String, String>> for Config {
    fn from(config: BTreeMap<String, String>) -> Self {
        let root_dirs: Vec<PathBuf> = match config.get("root_dirs") {
            Some(root_dirs) => parse_dirs(root_dirs),
            _ => vec![PathBuf::from(ROOT)]
        };
        let dirs: Vec<PathBuf> = match config.get("dirs") {
            Some(dirs) => parse_dirs(dirs),
            _ => vec![]
        };
        let layout = match config.get("session_layout") {
            Some(layout) => parse_layout(layout),
            _ => LayoutInfo::BuiltIn("default".to_string())
        };
        let root_files = match config.get("root_files") {
            Some(root_files) => parse_root_files(root_files),
            _ => vec![".git".to_string()]
        };
        Self {
            root_dirs,
            dirs,
            layout,
            root_files
        }
    }
}

