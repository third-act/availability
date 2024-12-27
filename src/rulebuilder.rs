use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::rule::Rule;
use crate::weekdays::{FRIDAY, MONDAY, SATURDAY, SUNDAY, THURSDAY, TUESDAY, WEDNESDAY};

pub struct RuleBuilder<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    // Store the raw strings instead of parsed times
    start_str: Option<String>,
    end_str: Option<String>,
    weekdays: Option<u8>,
    off: bool,
    payload: Option<T>,
}

impl<T> RuleBuilder<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    /// Create a new builder with no data set yet.
    pub fn new() -> Self {
        RuleBuilder {
            start_str: None,
            end_str: None,
            weekdays: None,
            off: false,
            payload: None,
        }
    }

    /// Store the raw start time string; no validation is done yet.
    /// Example: "231225093000" for 2023-12-25 09:30:00.
    pub fn start_time_str(mut self, datetime_str: &str) -> Self {
        self.start_str = Some(datetime_str.to_string());
        self
    }

    /// Store the raw end time string; no validation is done yet.
    /// Example: "231225173000" for 2023-12-25 17:30:00.
    pub fn end_time_str(mut self, datetime_str: &str) -> Self {
        self.end_str = Some(datetime_str.to_string());
        self
    }

    /// Set the start time using a NaiveDateTime directly
    pub fn start_datetime(mut self, datetime: NaiveDateTime) -> Self {
        // Convert the datetime to the expected string format
        let datetime_str = datetime.format("%y%m%d%H%M%S").to_string();
        self.start_str = Some(datetime_str);
        self
    }

    /// Set the end time using a NaiveDateTime directly
    pub fn end_datetime(mut self, datetime: NaiveDateTime) -> Self {
        // Convert the datetime to the expected string format
        let datetime_str = datetime.format("%y%m%d%H%M%S").to_string();
        self.end_str = Some(datetime_str);
        self
    }

    /// Bulk-set weekdays from string slice array.
    ///
    /// If **any** string is invalid, we set a special sentinel bit pattern
    /// (`0xFF`) to indicate an invalid weekday was encountered. Then in
    /// `build()` we detect `Some(0xFF)` and return an error.
    pub fn weekdays(mut self, days: &[&str]) -> Self {
        let mut mask = self.weekdays.unwrap_or(0);
        for day in days {
            match day.to_lowercase().as_str() {
                "monday" | "mon" => mask |= MONDAY,
                "tuesday" | "tue" => mask |= TUESDAY,
                "wednesday" | "wed" => mask |= WEDNESDAY,
                "thursday" | "thu" => mask |= THURSDAY,
                "friday" | "fri" => mask |= FRIDAY,
                "saturday" | "sat" => mask |= SATURDAY,
                "sunday" | "sun" => mask |= SUNDAY,
                _ => {
                    // Sentinel for "invalid weekday"
                    mask = 0xFF;
                    // We can break now, because we only need to record
                    // that at least one weekday was invalid.
                    break;
                }
            };
        }
        self.weekdays = Some(mask);
        self
    }

    pub fn monday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | MONDAY;
        self.weekdays = Some(val);
        self
    }

    pub fn tuesday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | TUESDAY;
        self.weekdays = Some(val);
        self
    }

    pub fn wednesday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | WEDNESDAY;
        self.weekdays = Some(val);
        self
    }

    pub fn thursday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | THURSDAY;
        self.weekdays = Some(val);
        self
    }

    pub fn friday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | FRIDAY;
        self.weekdays = Some(val);
        self
    }

    pub fn saturday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | SATURDAY;
        self.weekdays = Some(val);
        self
    }

    pub fn sunday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | SUNDAY;
        self.weekdays = Some(val);
        self
    }

    /// Store whether the rule is "off" (closed).
    pub fn off(mut self, off: bool) -> Self {
        self.off = off;
        self
    }

    /// Store the payload directly.
    pub fn payload(mut self, payload: T) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Parse/validate everything and return `Rule<T>` or an error.
    /// This is where all actual parsing logic and validations happen.
    pub fn build(self) -> Result<Rule<T>, String> {
        // First, ensure we had a start/end string
        let start_str = self
            .start_str
            .ok_or("Start time is required and was never set")?;
        let end_str = self
            .end_str
            .ok_or("End time is required and was never set")?;

        // Validate they are each 12 chars
        if start_str.len() != 12 {
            return Err(format!("Invalid start time format: {}", start_str));
        }
        if end_str.len() != 12 {
            return Err(format!("Invalid end time format: {}", end_str));
        }

        // Parse them both
        let start =
            parse_datetime(&start_str).map_err(|e| format!("Error parsing start: {}", e))?;
        let end = parse_datetime(&end_str).map_err(|e| format!("Error parsing end: {}", e))?;

        // Additional validation: ensure start < end
        if start >= end {
            return Err("Start must not be after or equal to end".into());
        }

        // Weekday check: 0xFF => we encountered an invalid weekday in `.weekdays()`
        if self.weekdays == Some(0xFF) {
            return Err("Invalid weekday encountered.".into());
        }

        // If all is good, build the actual `Rule`
        Rule::new(start, end, self.weekdays, self.off, self.payload)
    }
}

/// Helper function to parse a 12-char datetime string of form "YYMMDDHHMMSS"
fn parse_datetime(datetime_str: &str) -> Result<NaiveDateTime, String> {
    let year = format!("20{}", &datetime_str[0..2])
        .parse::<i32>()
        .map_err(|_| "Invalid year")?;
    let month = datetime_str[2..4]
        .parse::<u32>()
        .map_err(|_| "Invalid month")?;
    let day = datetime_str[4..6]
        .parse::<u32>()
        .map_err(|_| "Invalid day")?;
    let hour = datetime_str[6..8]
        .parse::<u32>()
        .map_err(|_| "Invalid hour")?;
    let minute = datetime_str[8..10]
        .parse::<u32>()
        .map_err(|_| "Invalid minute")?;
    let second = datetime_str[10..12]
        .parse::<u32>()
        .map_err(|_| "Invalid second")?;

    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .ok_or("Invalid date".to_string())?
        .and_hms_opt(hour, minute, second)
        .ok_or("Invalid time".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use serde_json::json;

    #[test]
    fn test_builder_basic_functionality() {
        let rule = RuleBuilder::<String>::new()
            .start_time_str("240101090000") // 2024-01-01 09:00:00
            .end_time_str("240101170000") // 2024-01-01 17:00:00
            .build()
            .unwrap();

        assert_eq!(
            rule.start.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-01-01 09:00:00"
        );
        assert_eq!(
            rule.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-01-01 17:00:00"
        );
        assert!(rule.weekdays.is_none());
        assert!(!rule.off);
        assert!(rule.payload.is_none());
    }

    #[test]
    fn test_builder_with_datetime() {
        let start =
            NaiveDateTime::parse_from_str("2024-01-01 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2024-01-01 17:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let rule = RuleBuilder::<String>::new()
            .start_datetime(start)
            .end_datetime(end)
            .build()
            .unwrap();

        assert_eq!(rule.start, start);
        assert_eq!(rule.end, end);
    }

    #[test]
    fn test_builder_weekdays_bulk() {
        let rule = RuleBuilder::<String>::new()
            .start_time_str("240101090000")
            .end_time_str("240101170000")
            .weekdays(&["monday", "wednesday", "friday"])
            .build()
            .unwrap();

        assert_eq!(rule.weekdays, Some(MONDAY | WEDNESDAY | FRIDAY));

        // Test with short forms
        let rule = RuleBuilder::<String>::new()
            .start_time_str("240101090000")
            .end_time_str("240101170000")
            .weekdays(&["mon", "wed", "fri"])
            .build()
            .unwrap();

        assert_eq!(rule.weekdays, Some(MONDAY | WEDNESDAY | FRIDAY));
    }

    #[test]
    fn test_builder_individual_weekdays() {
        let rule = RuleBuilder::<String>::new()
            .start_time_str("240101090000")
            .end_time_str("240101170000")
            .monday()
            .wednesday()
            .friday()
            .build()
            .unwrap();

        assert_eq!(rule.weekdays, Some(MONDAY | WEDNESDAY | FRIDAY));
    }

    #[test]
    fn test_builder_with_payload() {
        let payload = json!({"status": "active"});
        let rule = RuleBuilder::new()
            .start_time_str("240101090000")
            .end_time_str("240101170000")
            .payload(payload.clone())
            .build()
            .unwrap();

        assert_eq!(rule.payload.unwrap(), payload);
    }

    #[test]
    fn test_builder_with_off() {
        let rule = RuleBuilder::<String>::new()
            .start_time_str("240101090000")
            .end_time_str("240101170000")
            .off(true)
            .build()
            .unwrap();

        assert!(rule.off);
    }

    #[test]
    fn test_builder_invalid_weekdays() {
        let result = RuleBuilder::<String>::new()
            .start_time_str("240101090000")
            .end_time_str("240101170000")
            .weekdays(&["monday", "invalid_day"])
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid weekday encountered.");
    }

    #[test]
    fn test_parse_datetime() {
        // Test valid datetime
        let result = parse_datetime("240101090000");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-01-01 09:00:00"
        );

        // Test invalid cases
        assert!(parse_datetime("invalid").is_err()); // Too short
        assert!(parse_datetime("240001090000").is_err()); // Invalid month
        assert!(parse_datetime("240132090000").is_err()); // Invalid day
        assert!(parse_datetime("240101250000").is_err()); // Invalid hour
        assert!(parse_datetime("240101096000").is_err()); // Invalid minute
        assert!(parse_datetime("240101090060").is_err()); // Invalid second
    }

    #[test]
    fn test_builder_validation_errors() {
        // Missing start time
        let result = RuleBuilder::<String>::new()
            .end_time_str("240101170000")
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Start time is required and was never set"
        );

        // Missing end time
        let result = RuleBuilder::<String>::new()
            .start_time_str("240101090000")
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "End time is required and was never set"
        );

        // Invalid datetime format (too short)
        let result = RuleBuilder::<String>::new()
            .start_time_str("2401")
            .end_time_str("240101170000")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid start time format"));

        // End before start
        let result = RuleBuilder::<String>::new()
            .start_time_str("240101170000")
            .end_time_str("240101090000")
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Start must not be after or equal to end"
        );

        // Equal start and end
        let result = RuleBuilder::<String>::new()
            .start_time_str("240101090000")
            .end_time_str("240101090000")
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Start must not be after or equal to end"
        );
    }

    #[test]
    fn test_idempotent_weekday_setting() {
        // Test single day multiple times
        let rule = RuleBuilder::<String>::new()
            .start_time_str("240101090000")
            .end_time_str("240101170000")
            .saturday()
            .saturday()
            .saturday()
            .build()
            .unwrap();

        // Should only have SATURDAY bit set once
        assert_eq!(rule.weekdays, Some(SATURDAY));

        // Test multiple days with repetition
        let rule = RuleBuilder::<String>::new()
            .start_time_str("240101090000")
            .end_time_str("240101170000")
            .monday()
            .wednesday()
            .monday() // Repeated monday
            .friday()
            .wednesday() // Repeated wednesday
            .build()
            .unwrap();

        // Should have exactly these three days set
        assert_eq!(rule.weekdays, Some(MONDAY | WEDNESDAY | FRIDAY));
    }
}
