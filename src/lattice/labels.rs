use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum Clearance {
    Public,
    Restricted,
    Confidential,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SecurityLabel {
    pub clearance: Clearance,
    pub compartments: BTreeSet<String>,
}

impl SecurityLabel {
    pub fn new(clearance: Clearance, compartments: BTreeSet<String>) -> Self {
        SecurityLabel {
            clearance,
            compartments,
        }
    }
}
