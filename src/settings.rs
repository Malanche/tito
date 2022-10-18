extern crate serde;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{Proposal, Scenario, Language, LanguageSettings};

#[derive(Clone)]
pub struct Competitor {
    pub id: String,
    pub files: String,
    pub result: Option<String>
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Evaluation {
    Grade {
        score: f64
    },
    RunError,
    NoFile
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub proposals: HashMap<String, Proposal>,
    pub language_settings: Option<HashMap<Language, LanguageSettings>>
}

impl Settings {
    /// Generates an example configuration for a problem written in shell
    pub fn example() -> Settings {
        Settings {
            proposals: vec![("problem-a".to_string(), Proposal {
                scenarios: vec![Scenario {
                    arguments: Some(vec!["Tito".to_string()]),
                    input: None,
                    output: None,
                    max_time: 1.0,
                    max_ram: None,
                    points: 10
                }],
                solution: "./problem-a.sh".to_string(),
                language: Language::Shell,
                points: 10
            })].into_iter().collect(),
            language_settings: Some(vec![(Language::Shell, Language::Shell.default_settings())].into_iter().collect())
        }
    }
}