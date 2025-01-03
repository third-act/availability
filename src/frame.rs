use std::fmt;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Frame<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub off: bool,
    pub payload: Option<T>,
}

impl<T> fmt::Display for Frame<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.off { "Off" } else { "On" };
        let payload_str = match &self.payload {
            Some(payload) => match serde_json::to_string(payload) {
                Ok(s) => s,
                Err(_) => "<invalid payload>".to_string(),
            },
            None => "None".to_string(),
        };
        write!(
            f,
            "Frame [Start: {}, End: {}, Status: {}, Payload: {}]",
            self.start, self.end, status, payload_str
        )
    }
}

impl<T> Frame<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub(crate) fn new(
        start: NaiveDateTime,
        end: NaiveDateTime,
        off: bool,
        payload: Option<T>,
    ) -> Self {
        Frame {
            start,
            end,
            off,
            payload,
        }
    }

    pub fn start_datetime(&self) -> NaiveDateTime {
        self.start
    }

    pub fn end_datetime(&self) -> NaiveDateTime {
        self.end
    }

    pub fn is_on(&self) -> bool {
        !self.off
    }

    pub fn is_off(&self) -> bool {
        self.off
    }

    pub fn payload(&self) -> Option<T> {
        self.payload.clone()
    }
}
