use std::fmt;

use chrono::{Duration, NaiveDateTime};
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
        let duration = self.duration();
        let total_seconds = duration.num_seconds();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        write!(
            f,
            "Frame spans {:02}:{:02}:{:02} [Start: {}, End: {}, Status: {}, Payload: {}]",
            hours, minutes, seconds, self.start, self.end, status, payload_str
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

    pub fn duration(&self) -> Duration {
        self.end.signed_duration_since(self.start)
    }
}
