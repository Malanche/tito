extern crate serde;

use serde::{Serialize, Deserialize};
use crate::{Language};

/// Describes what surrounds the execution of the solution of a problem (or the attempt)
#[derive(Serialize, Deserialize, Clone)]
pub struct Scenario {
    /// Arguments provided at execution time
    pub arguments: Option<Vec<String>>,
    /// Std input for the program
    pub input: Option<String>,
    /// Expected output
    pub output: Option<String>,
    /// Maximum allowed time
    pub max_time: f32,
    /// Maximum allowed Ram
    pub max_ram: Option<u32>,
    /// Points that this scenario gives
    pub points: u32
}

/// Describes a problem, which has multiple scenarios and a certain language
#[derive(Serialize, Deserialize, Clone)]
pub struct Problem {
    /// Scenarios where the problem should run
    pub scenarios: Vec<Scenario>,
    /// Filename, wthout extension, for the problem
    pub filename: String,
    /// If this is present, a specific filename is searched for
    pub language: Option<Language>,
    /// Total points that the problem gives
    pub points: u32
}

/// Describes a problem, which has multiple scenarios and a certain language
#[derive(Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub scenarios: Vec<Scenario>,
    pub solution: String,
    pub language: Language,
    pub points: u32
}