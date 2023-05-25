use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Emergency {
    pub keyword: String,
    pub number: i64
}