use bitflags::bitflags;
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct Weekdays: u32 {
        const ALL                         = 0b1111111;
        const MONDAY                      = 0b0000001;
        const TUESDAY                     = 0b0000010;
        const WEDNESDAY                   = 0b0000100;
        const THURSDAY                    = 0b0001000;
        const FRIDAY                      = 0b0010000;
        const SATURDAY                    = 0b0100000;
        const SUNDAY                      = 0b1000000;
    }
}

impl Weekdays {
    pub fn from_chrono_weekday(weekday: chrono::Weekday) -> Self {
        match weekday {
            chrono::Weekday::Mon => Self::MONDAY,
            chrono::Weekday::Tue => Self::TUESDAY,
            chrono::Weekday::Wed => Self::WEDNESDAY,
            chrono::Weekday::Thu => Self::THURSDAY,
            chrono::Weekday::Fri => Self::FRIDAY,
            chrono::Weekday::Sat => Self::SATURDAY,
            chrono::Weekday::Sun => Self::SUNDAY,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rule<T: Serialize> {
    pub start_date: NaiveDate,

    pub end_date: NaiveDate,

    pub start_time: NaiveTime,

    pub end_time: NaiveTime,

    #[serde(with = "integer_representation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub weekdays: Option<Weekdays>,

    pub state: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub payload: Option<T>,
}

mod integer_representation {
    use super::Weekdays;
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
    type IntRep = u32;
    type Flags = Option<Weekdays>;

    pub fn serialize<S>(flags: &Flags, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(flags) = flags {
            flags.bits().serialize(serializer)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Flags, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw: IntRep = IntRep::deserialize(deserializer)?;
        Ok(Some(Weekdays::from_bits(raw).ok_or(
            serde::de::Error::custom(format!("Unexpected flags value {}", raw)),
        )?))
    }
}
