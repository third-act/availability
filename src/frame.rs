use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Frame<T: Clone> {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub state: T,
}
