use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::models::Builder::Builder;

struct RadioIdentifier {
    pub org: String,
    pub county: String,
    pub agency: u64,
    pub car_type: u64,
    pub number: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Emergency {
    pub keyword: String,
    pub village: String,

    pub number: i64,
}

impl Builder<Emergency> for Emergency {
    type E = String;

    fn build(&self) -> Result<Emergency, Self::E> {
        return Ok(Emergency {
            keyword: String::from(""),
            number: 0,
        });
    }
}

impl FromStr for Emergency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return Ok(Emergency {
            keyword: String::from(""),
            number: 0,
        });
    }
}
