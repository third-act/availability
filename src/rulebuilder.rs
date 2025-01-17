use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::rule::Rule;
use crate::weekdays::{
    ALL_WEEKDAYS, FRIDAY, MONDAY, SATURDAY, SUNDAY, THURSDAY, TUESDAY, WEDNESDAY,
};

#[derive(Default)]
pub struct RuleBuilder<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
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
    /// Creates a new `RuleBuilder` instance with default values.
    /// All fields are initially None, except for `off`, which defaults to `false`.
    pub fn new() -> Self {
        RuleBuilder {
            start_str: None,
            end_str: None,
            weekdays: None,
            off: false,
            payload: None,
        }
    }

    /// Sets the start time of the rule using a raw datetime string.
    ///
    /// The datetime string must be in the `"YYMMDDHHMMSS"` format, representing
    /// year, month, day, hour, minute, and second. For example, `"231225093000"`
    /// corresponds to December 25, 2023, at 09:30:00.
    ///
    /// **Note:** This method does not perform validation on the datetime string.
    ///  Validation is done in the `build()` method which returns a `Result`.
    pub fn start_time_str(mut self, datetime_str: &str) -> Self {
        self.start_str = Some(datetime_str.to_string());
        self
    }

    /// Sets the end time of the rule using a raw datetime string.
    ///
    /// The datetime string must be in the `"YYMMDDHHMMSS"` format, representing
    /// year, month, day, hour, minute, and second. For example, `"231225173000"`
    /// corresponds to December 25, 2023, at 17:30:00.
    ///
    /// **Note:** This method does not perform validation on the datetime string.
    ///  Validation is done in the `build()` method which returns a `Result`.
    pub fn end_time_str(mut self, datetime_str: &str) -> Self {
        self.end_str = Some(datetime_str.to_string());
        self
    }

    /// Sets the start time of the rule using a `NaiveDateTime` instance.
    ///
    /// This method converts the provided `NaiveDateTime` into the expected
    /// `"YYMMDDHHMMSS"` string format internally.
    pub fn start_datetime(mut self, datetime: NaiveDateTime) -> Self {
        let datetime_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        self.start_str = Some(datetime_str);
        self
    }

    /// Sets the end time of the rule using a `NaiveDateTime` instance.
    ///
    /// This method converts the provided `NaiveDateTime` into the expected
    /// `"YYMMDDHHMMSS"` string format internally.
    pub fn end_datetime(mut self, datetime: NaiveDateTime) -> Self {
        let datetime_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        self.end_str = Some(datetime_str);
        self
    }

    /// Sets the weekdays on which the rule is active using a slice of string slices.
    ///
    /// Each string should represent a day of the week, such as `"monday"`, `"tue"`, etc.
    /// The method is case-insensitive and accepts both full names and common abbreviations.
    ///
    /// If **any** string in the slice is invalid (i.e., does not correspond to a valid weekday),
    /// the builder sets a special sentinel bit pattern (`0xFF`) to indicate the presence of an
    /// invalid weekday. The `build()` method will then detect this and return an error.
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
                    // Break early because at least one weekday was invalid.
                    break;
                }
            };
        }
        self.weekdays = Some(mask);
        self
    }

    /// Adds Monday to the set of active weekdays for the rule.
    pub fn monday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | MONDAY;
        self.weekdays = Some(val);
        self
    }
    /// Adds Tuesday to the set of active weekdays for the rule.
    pub fn tuesday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | TUESDAY;
        self.weekdays = Some(val);
        self
    }
    /// Adds Wednesday to the set of active weekdays for the rule.
    pub fn wednesday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | WEDNESDAY;
        self.weekdays = Some(val);
        self
    }
    /// Adds Thursday to the set of active weekdays for the rule.
    pub fn thursday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | THURSDAY;
        self.weekdays = Some(val);
        self
    }
    /// Adds Friday to the set of active weekdays for the rule.
    pub fn friday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | FRIDAY;
        self.weekdays = Some(val);
        self
    }
    /// Adds Saturday to the set of active weekdays for the rule.
    pub fn saturday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | SATURDAY;
        self.weekdays = Some(val);
        self
    }
    /// Adds Sunday to the set of active weekdays for the rule.
    pub fn sunday(mut self) -> Self {
        let val = self.weekdays.unwrap_or(0) | SUNDAY;
        self.weekdays = Some(val);
        self
    }
    /// Adds all weekdays to the set of active weekdays for the rule.
    pub fn all_weekdays(mut self) -> Self {
        self.weekdays = Some(ALL_WEEKDAYS);
        self
    }

    /// Sets whether the rule is "off" or "on".
    ///
    /// - `true`: The rule is "off" (closed).
    /// - `false`: The rule is "on" (active).
    ///
    /// # Parameters
    ///
    /// - `off`: A boolean indicating the status of the rule
    pub fn off(mut self, off: bool) -> Self {
        self.off = off;
        self
    }

    /// Attaches a custom payload to the rule.
    ///
    /// The payload can be any type that implements `Serialize`, `Deserialize`, and `Clone`.
    ///
    /// # Parameters
    ///
    /// - `payload`: The payload to attach to the rule.
    ///
    /// # Returns
    ///
    /// Returns the updated `RuleBuilder` instance.
    pub fn payload(mut self, payload: T) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Builds the `Rule` instance by parsing and validating all set fields.
    ///
    /// This method performs the following steps:
    /// 1. Ensures that both start and end times are set.
    /// 2. Validates the format and correctness of the datetime strings.
    /// 3. Checks that the start time precedes the end time.
    /// 4. Validates the weekdays, ensuring no invalid weekdays were set.
    ///
    /// If all validations pass, it returns an `Ok(Rule<T>)`. Otherwise, it returns an `Err(String)`
    /// containing an error message.
    ///
    /// # Errors
    ///
    /// - Returns an error if either the start or end time is not set.
    /// - Returns an error if the datetime strings are improperly formatted or invalid.
    /// - Returns an error if the start time is not before the end time.
    /// - Returns an error if invalid weekdays were specified.
    ///
    /// # Returns
    ///
    /// - `Ok(Rule<T>)` if the rule is successfully built.
    /// - `Err(String)` containing an error message if validation fails.
    pub fn build(self) -> Result<Rule<T>, String> {
        // First, ensure we had a start/end string
        let start_str = self
            .start_str
            .ok_or("Start time is required and was never set")?;
        let end_str = self
            .end_str
            .ok_or("End time is required and was never set")?;

        // Validate they are each 12 chars
        if !start_str.contains('-') || start_str.len() != 19 {
            return Err(format!(
                "Invalid start time format: {}. Expected format: YYYY-MM DD-HH:MM:SS",
                start_str
            ));
        }
        if !end_str.contains('-') || end_str.len() != 19 {
            return Err(format!(
                "Invalid end time format: {}. Expected format: YYYY-MM-DD HH:MM:SS",
                end_str
            ));
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

/// Helper function to parse a datetime string in the form "YYYY-MM-DD HH:MM:SS"
fn parse_datetime(datetime_str: &str) -> Result<NaiveDateTime, chrono::ParseError> {
    NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
}

#[cfg(test)]
mod tests {
    use crate::weekdays::get_days_from_mask;

    use super::*;
    use chrono::NaiveDateTime;
    use serde_json::json;

    #[test]
    fn test_builder_basic_functionality() {
        let rule = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 09:00:00") // 2024-01-01 09:00:00
            .end_time_str("2024-01-01 17:00:00") // 2024-01-01 17:00:00
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
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 17:00:00")
            .weekdays(&["monday", "wednesday", "friday"])
            .build()
            .unwrap();

        assert_eq!(rule.weekdays, Some(MONDAY | WEDNESDAY | FRIDAY));

        // Test with short forms
        let rule = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 17:00:00")
            .weekdays(&["mon", "wed", "fri"])
            .build()
            .unwrap();

        assert_eq!(rule.weekdays, Some(MONDAY | WEDNESDAY | FRIDAY));
    }

    #[test]
    fn test_builder_individual_weekdays() {
        let rule = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 17:00:00")
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
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 17:00:00")
            .payload(payload.clone())
            .build()
            .unwrap();

        assert_eq!(rule.payload.unwrap(), payload);
    }

    #[test]
    fn test_builder_with_off() {
        let rule = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 17:00:00")
            .off(true)
            .build()
            .unwrap();

        assert!(rule.off);
    }

    #[test]
    fn test_builder_invalid_weekdays() {
        let result = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 17:00:00")
            .weekdays(&["monday", "invalid_day"])
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid weekday encountered.");
    }

    #[test]
    fn test_parse_datetime() {
        // Test valid datetime
        let result = parse_datetime("2024-01-01 09:00:00");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-01-01 09:00:00"
        );

        // Test invalid cases
        assert!(parse_datetime("invalid").is_err()); // Too short
        assert!(parse_datetime("2024-00-01 09:00:00").is_err()); // Invalid month
        assert!(parse_datetime("2024-01-32 09:00:00").is_err()); // Invalid day
        assert!(parse_datetime("2024-01-01 25:00:00").is_err()); // Invalid hour
        assert!(parse_datetime("2024-01-01 09:60:00").is_err()); // Invalid minute
        assert!(parse_datetime("2024-01-01 09:00:61").is_err()); // Invalid second
        assert!(!parse_datetime("2024-01-01 09:00:60").is_err()); // Chrono allows leap seconds
    }

    #[test]
    fn test_builder_validation_errors() {
        // Missing start time
        let result = RuleBuilder::<String>::new()
            .end_time_str("2024-01-01 17:00:00")
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Start time is required and was never set"
        );

        // Missing end time
        let result = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 09:00:00")
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "End time is required and was never set"
        );

        // Invalid datetime format (too short)
        let result = RuleBuilder::<String>::new()
            .start_time_str("2401")
            .end_time_str("2024-01-01 17:00:00")
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid start time format"));

        // End before start
        let result = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 17:00:00")
            .end_time_str("2024-01-01 09:00:00")
            .build();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Start must not be after or equal to end"
        );

        // Equal start and end
        let result = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 09:00:00")
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
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 17:00:00")
            .saturday()
            .saturday()
            .saturday()
            .build()
            .unwrap();

        // Should only have SATURDAY bit set once
        assert_eq!(rule.weekdays, Some(SATURDAY));

        // Test multiple days with repetition
        let rule = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 17:00:00")
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
    #[test]

    fn test_builder_all_weekdays() {
        let rule = RuleBuilder::<String>::new()
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-01 17:00:00")
            .all_weekdays()
            .build()
            .unwrap();

        assert_eq!(rule.weekdays, Some(ALL_WEEKDAYS));

        // Verify that all days are set using get_days_from_mask
        assert_eq!(
            get_days_from_mask(rule.weekdays.unwrap()),
            vec![
                "monday",
                "tuesday",
                "wednesday",
                "thursday",
                "friday",
                "saturday",
                "sunday"
            ]
        );
    }
}
