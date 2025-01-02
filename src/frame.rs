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
