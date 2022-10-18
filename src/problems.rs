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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Maximum allowed time, in seconds
    pub max_time: f32,
    /// Maximum allowed Ram, in bytes
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
    /// Input scenarios
    pub scenarios: Vec<Scenario>,
    /// Path to the solution of the problem
    pub solution: String,
    /// Language the problem is written in
    pub language: Language,
    /// Number of points this problem gives
    pub points: u32
}