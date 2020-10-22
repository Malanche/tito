extern crate serde;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Tool {
    /// Name of the utility
    pub utility: String,
    /// Indicates if the tool is temporal and not system-wide available (for example resulting binaries)
    pub temporal: bool,
    /// Arguments that the utility receives
    pub arguments: Vec<String>
}

/// Languages available
#[derive(Serialize, Deserialize, PartialEq, Hash, Clone)]
pub enum Language {
    Rust,
    Shell,
    Cpp,
    C,
    Python2,
    Python3
}

impl Eq for Language{}

#[derive(Serialize, Deserialize, Clone)]
pub struct LanguageSettings {
    /// Tools to be executed before meassuring, i.e. compilers
    pub pre_tools: Option<Vec<Tool>>,
    /// Tool to be executed, i.e. the binary or interpreter with given source file
    pub tool: Tool,
    /// Extension of the files
    pub extension: String
}

impl LanguageSettings {
    pub fn default(language: Language) -> LanguageSettings {
        match language {
            Language::Rust => {
                let pre_tools = Some(vec![Tool{utility: "rustc".into(), temporal: false, arguments: vec!["-o".into(), "a.exe".into(), "{filename}".into()]}]);
                let tool = Tool{utility: "{pwd}/a.exe".into(), temporal: true, arguments: vec![]};
                LanguageSettings {
                    pre_tools,
                    tool,
                    extension: "rs".into()
                }
            },
            Language::Shell => {
                let tool = Tool{utility: "bash".into(), temporal: false, arguments: vec!["{filename}".into()]};
                LanguageSettings {
                    pre_tools: None,
                    tool,
                    extension: "sh".into()
                }
            },
            Language::Cpp => {
                let pre_tools = Some(vec![Tool{utility: "g++".into(), temporal: false, arguments: vec!["-o".into(), "a.exe".into(), "{filename}".into()]}]);
                let tool = Tool{utility: "{pwd}/a.exe".into(), temporal: true, arguments: vec![]};
                LanguageSettings {
                    pre_tools,
                    tool,
                    extension: "cpp".into()
                }
            },
            Language::C => {
                let pre_tools = Some(vec![Tool{utility: "gcc".into(), temporal: false, arguments: vec!["-o".into(), "a.exe".into(), "{filename}".into()]}]);
                let tool = Tool{utility: "{pwd}/a.exe".into(), temporal: true, arguments: vec![]};
                LanguageSettings {
                    pre_tools,
                    tool,
                    extension: "c".into()
                }
            },
            Language::Python2 => {
                let tool = if(cfg!(target_os = "windows")) {
                    Tool{utility: "python".into(), temporal: false, arguments: vec!["{filename}".into()]}
                } else {
                    Tool{utility: "python2".into(), temporal: false, arguments: vec!["{filename}".into()]}
                };
                LanguageSettings {
                    pre_tools: None,
                    tool,
                    extension: "py".into()
                }
            },
            Language::Python3 => {
                let tool = if(cfg!(target_os = "windows")) {
                    Tool{utility: "python".into(), temporal: false, arguments: vec!["{filename}".into()]}
                } else {
                    Tool{utility: "python3".into(), temporal: false, arguments: vec!["{filename}".into()]}
                };
                LanguageSettings {
                    pre_tools: None,
                    tool,
                    extension: "py".into()
                }
            }
        }
    }
}