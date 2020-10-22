extern crate serde;

use std::collections::HashMap;
use crate::Problem;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Arena {
    pub problems: HashMap<String, Problem>
}