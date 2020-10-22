extern crate serde;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{Proposal, Language, LanguageSettings};

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