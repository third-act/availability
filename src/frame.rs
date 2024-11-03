use crate::util;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Frame<T: Serialize> {
    pub start: NaiveDateTime,

    pub end: NaiveDateTime,

    pub state: bool,

    #[serde(skip_serializing_if = "util::is_none")]
    #[serde(default)]
    pub payload: Option<T>,
}
