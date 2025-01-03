use std::fmt;

use chrono::{naive, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

use crate::{
    crate_parameters::{BASE_RULE_YEAR_END, BASE_RULE_YEAR_START},
    weekdays::{
        get_days_from_mask, FRIDAY, MONDAY, SATURDAY, SUNDAY, THURSDAY, TUESDAY, WEDNESDAY,
    },
};

#[derive(Debug, Clone)]
pub struct Rule<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub weekdays: Option<u8>,
    pub off: bool,
    pub payload: Option<T>,
}

impl<T> fmt::Display for Rule<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.off { "Off" } else { "On" };
        let weekdays_str = match self.weekdays {
            Some(mask) => {
                let days = get_days_from_mask(mask);
                if days.is_empty() {
                    "All Days".to_string()
                } else {
                    days.join(", ")
                }
            }
            None => "No Specific Days".to_string(),
        };
        let payload_str = match &self.payload {
            Some(payload) => match serde_json::to_string(payload) {
                Ok(s) => s,
                Err(_) => "<invalid payload>".to_string(),
            },
            None => "None".to_string(),
        };
        write!(
            f,
            "Rule [Start: {}, End: {}, Weekdays: {}, Status: {}, Payload: {}]",
            self.start, self.end, weekdays_str, status, payload_str
        )
    }
}

impl<T> Rule<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub(crate) fn new(
        start: NaiveDateTime,
        end: NaiveDateTime,
        weekdays: Option<u8>,
        off: bool,
        payload: Option<T>,
    ) -> Result<Self, String> {
        if start >= end {
            return Err("Start must not be before after or equal to end".to_string());
        }
        Ok(Rule {
            start,
            end,
            weekdays,
            off,
            payload,
        })
    }

    /// Check if rule is active at the given NaiveDateTime.
    pub fn is_active(&self, date_time: NaiveDateTime) -> bool {
        match self.is_absolute() {
            true => self.is_date_time_within(date_time) && self.is_time_within(date_time.time()),
            false => {
                if self.is_weekday_enabled(date_time) {
                    if self.off {
                        false
                    } else {
                        self.is_date_time_within(date_time) && self.is_time_within(date_time.time())
                    }
                } else {
                    false
                }
            }
        }
    }

    /// True if rule is open at the given NaiveDateTime.
    /// Interanlly checks if the rule is absolute or relative and if the date and time are within the rule.
    pub fn is_open(&self, date_time: NaiveDateTime) -> bool {
        if self.off {
            return false;
        }
        self.is_active(date_time)
    }

    /// Check if two rules overlap in NaiveDateTime.
    pub(crate) fn datetime_overlaps_with(&self, other: &Rule<T>) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Check the day of date_time and returns true if the weekday is enabled in the rule.
    pub fn is_weekday_enabled(&self, date_time: NaiveDateTime) -> bool {
        let weekday_mask = match date_time.weekday() {
            Weekday::Mon => MONDAY,
            Weekday::Tue => TUESDAY,
            Weekday::Wed => WEDNESDAY,
            Weekday::Thu => THURSDAY,
            Weekday::Fri => FRIDAY,
            Weekday::Sat => SATURDAY,
            Weekday::Sun => SUNDAY,
        };

        self.weekdays
            .map(|enabled_days| enabled_days & weekday_mask != 0)
            .unwrap_or(false)
    }

    /// True if rule is absolute (i.e. it has not weekdays)
    pub fn is_absolute(&self) -> bool {
        match self.weekdays {
            Some(weekdays) => weekdays == 0,
            None => true,
        }
    }

    /// True if rule is relative (i.e. it has weekdays)
    pub fn is_relative(&self) -> bool {
        match self.weekdays {
            Some(weekdays) => weekdays != 0,
            None => false,
        }
    }

    /// True if NaiveDateTime is within entire range of rule.
    /// Eg. 2024-01-05 06:00:00 is within 2024-01-01 09:00:00 to 2024-01-31 17:00:00
    /// despite not being within the time range.
    pub fn is_date_time_within(&self, time: NaiveDateTime) -> bool {
        time >= self.start && time < self.end
    }

    /// True if NaiveTime is within the time range of the rule.
    /// Eg. 2024-01-01 06:00:00 is not within 2024-01-01 09:00:00 to 2024-01-01 17:00:00
    pub fn is_time_within(&self, time: NaiveTime) -> bool {
        time >= self.start.time() && time < self.end.time()
    }

    /// Base rule is always off and covers the entire range of possible dates.
    /// This has lowest priority and is not modifiable by user.
    pub(crate) fn base_rule() -> Rule<T> {
        let naive_date_start = NaiveDate::from_ymd_opt(BASE_RULE_YEAR_START, 1, 1).unwrap();
        let naive_date_end = NaiveDate::from_ymd_opt(BASE_RULE_YEAR_END, 1, 1).unwrap();
        let naive_time = naive::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        Rule {
            start: NaiveDateTime::new(naive_date_start, naive_time),
            end: NaiveDateTime::new(naive_date_end, naive_time),
            weekdays: None,
            off: true,
            payload: None,
        }
    }

    pub fn has_matching_payload(&self, other: &Rule<T>) -> Result<bool, serde_json::Error> {
        match (&self.payload, &other.payload) {
            (None, None) => Ok(true),
            (None, Some(_)) | (Some(_), None) => Ok(false),
            (Some(payload1), Some(payload2)) => {
                let value1 = serde_json::to_value(payload1)?;
                let value2 = serde_json::to_value(payload2)?;

                Ok(value1 == value2)
            }
        }
    }

    /// True if any of the weekdays in self are present in other.
    /// Eg. 0b00000001 (Monday) is present in 0b00000111 (Monday, Tuesday, Wednesday)
    pub fn has_weekdays_in(&self, other: &Rule<T>) -> bool {
        match (self.weekdays, other.weekdays) {
            (None, _) | (_, None) => false,
            (Some(self_days), Some(other_days)) => {
                // Check if any of the weekdays in self are present in other
                self_days & other_days != 0
            }
        }
    }
}

/// Split relative rule to several absolute rules because they can easily be converted to frames.
pub(crate) fn relative_to_absolute_rules<T>(rule: Rule<T>) -> Result<Vec<Rule<T>>, String>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    if rule.is_absolute() {
        return Ok(vec![rule]);
    }
    if rule.start.date() == rule.end.date() {
        return Err("Rule spans only one day and cannot be divided further".to_string());
    }

    // Split rule into several rules that span only one day
    let mut absolute_rules: Vec<Rule<T>> = Vec::new();
    let mut current_day = rule.start.date();

    while current_day <= rule.end.date() {
        let current_datetime = current_day.and_hms_opt(0, 0, 0).unwrap();

        if rule.is_weekday_enabled(current_datetime) {
            let start_time = rule.start.time();

            // Create the end time for this day
            let end_time = rule.end.time();

            let start = current_day.and_time(start_time);
            let end = current_day.and_time(end_time);

            let new_rule = Rule::new(
                start,
                end,
                None, // Convert to absolute rule
                rule.off,
                rule.payload.clone(),
            )?;

            absolute_rules.push(new_rule);
        }

        current_day = current_day.succ_opt().unwrap();
    }

    Ok(absolute_rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::naive::NaiveTime;
    use serde_json::json;

    fn create_test_datetime(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> NaiveDateTime {
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(year, month, day).unwrap(),
            NaiveTime::from_hms_opt(hour, min, sec).unwrap(),
        )
    }

    #[test]
    fn test_rule_creation() {
        let start = create_test_datetime(2024, 1, 1, 9, 0, 0);
        let end = create_test_datetime(2024, 1, 1, 17, 0, 0);

        // Valid rule creation
        let rule = Rule::<String>::new(start, end, None, false, None);
        assert!(rule.is_ok());

        // Invalid rule with end before start
        let invalid_rule = Rule::<String>::new(end, start, None, false, None);
        assert!(invalid_rule.is_err());

        // Invalid rule with equal start and end
        let invalid_rule = Rule::<String>::new(start, start, None, false, None);
        assert!(invalid_rule.is_err());
    }

    #[test]
    fn test_absolute_rule_is_open() {
        let start = create_test_datetime(2024, 1, 1, 9, 0, 0);
        let end = create_test_datetime(2024, 1, 1, 17, 0, 0);
        let rule = Rule::<String>::new(start, end, None, false, None).unwrap();

        // Test times within range
        assert!(rule.is_open(create_test_datetime(2024, 1, 1, 9, 0, 0)));
        assert!(rule.is_open(create_test_datetime(2024, 1, 1, 12, 0, 0)));
        assert!(rule.is_open(create_test_datetime(2024, 1, 1, 16, 59, 59)));

        // Test times outside range
        assert!(!rule.is_open(create_test_datetime(2024, 1, 1, 8, 59, 59)));
        assert!(!rule.is_open(create_test_datetime(2024, 1, 1, 17, 0, 0)));
        assert!(!rule.is_open(create_test_datetime(2024, 1, 1, 17, 0, 1)));
    }

    #[test]
    fn test_relative_rule_is_open() {
        let start = create_test_datetime(2024, 1, 1, 9, 0, 0);
        let end = create_test_datetime(2024, 1, 5, 17, 0, 0);
        // Monday and Wednesday only
        let rule = Rule::<String>::new(start, end, Some(MONDAY | WEDNESDAY), false, None).unwrap();

        // Monday at valid time
        assert!(rule.is_open(create_test_datetime(2024, 1, 1, 12, 0, 0)));
        // Wednesday at valid time
        assert!(rule.is_open(create_test_datetime(2024, 1, 3, 12, 0, 0)));
        // Tuesday at valid time (should be closed)
        assert!(!rule.is_open(create_test_datetime(2024, 1, 2, 12, 0, 0)));
        // Monday at invalid time
        assert!(!rule.is_open(create_test_datetime(2024, 1, 1, 8, 0, 0)));
    }

    #[test]
    fn test_off_rule() {
        let start = create_test_datetime(2024, 1, 1, 9, 0, 0);
        let end = create_test_datetime(2024, 1, 1, 17, 0, 0);
        let rule = Rule::<String>::new(start, end, None, true, None).unwrap();

        // Should always be closed when off is true
        assert!(!rule.is_open(create_test_datetime(2024, 1, 1, 12, 0, 0)));
    }

    #[test]
    fn test_datetime_overlaps_with() {
        let rule1 = Rule::<String>::new(
            create_test_datetime(2024, 1, 1, 9, 0, 0),
            create_test_datetime(2024, 1, 1, 17, 0, 0),
            None,
            false,
            None,
        )
        .unwrap();

        // Completely overlapping
        let rule2 = Rule::<String>::new(
            create_test_datetime(2024, 1, 1, 9, 0, 0),
            create_test_datetime(2024, 1, 1, 17, 0, 0),
            None,
            false,
            None,
        )
        .unwrap();
        assert!(rule1.datetime_overlaps_with(&rule2));

        // Partially overlapping
        let rule3 = Rule::<String>::new(
            create_test_datetime(2024, 1, 1, 8, 0, 0),
            create_test_datetime(2024, 1, 1, 10, 0, 0),
            None,
            false,
            None,
        )
        .unwrap();
        assert!(rule1.datetime_overlaps_with(&rule3));

        // Non-overlapping
        let rule4 = Rule::<String>::new(
            create_test_datetime(2024, 1, 1, 17, 0, 0),
            create_test_datetime(2024, 1, 1, 18, 0, 0),
            None,
            false,
            None,
        )
        .unwrap();
        assert!(!rule1.datetime_overlaps_with(&rule4));
    }

    #[test]
    fn test_relative_to_absolute_rules() {
        let start = create_test_datetime(2024, 1, 1, 9, 0, 0); // Monday
        let end = create_test_datetime(2024, 1, 3, 17, 0, 0); // Wednesday
        let rule = Rule::<String>::new(start, end, Some(MONDAY | WEDNESDAY), false, None).unwrap();

        let absolute_rules = relative_to_absolute_rules(rule).unwrap();
        assert_eq!(absolute_rules.len(), 2); // Should create two rules (Monday and Wednesday)

        // Check first rule (Monday)
        assert_eq!(absolute_rules[0].start.date().weekday(), Weekday::Mon);
        assert_eq!(
            absolute_rules[0].start.time(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            absolute_rules[0].end.time(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap()
        );
        assert!(absolute_rules[0].is_absolute());

        // Check second rule (Wednesday)
        assert_eq!(absolute_rules[1].start.date().weekday(), Weekday::Wed);
        assert_eq!(
            absolute_rules[1].start.time(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap()
        );
        assert_eq!(
            absolute_rules[1].end.time(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap()
        );
        assert!(absolute_rules[1].is_absolute());
    }

    #[test]
    fn test_has_matching_payload() {
        let start = create_test_datetime(2024, 1, 1, 9, 0, 0);
        let end = create_test_datetime(2024, 1, 1, 17, 0, 0);

        // Test with JSON payloads
        let rule1 = Rule::new(start, end, None, false, Some(json!({"status": "active"}))).unwrap();
        let rule2 = Rule::new(start, end, None, false, Some(json!({"status": "active"}))).unwrap();
        let rule3 =
            Rule::new(start, end, None, false, Some(json!({"status": "inactive"}))).unwrap();

        assert!(rule1.has_matching_payload(&rule2).unwrap());
        assert!(!rule1.has_matching_payload(&rule3).unwrap());

        // Test with None payloads
        let rule4 = Rule::<serde_json::Value>::new(start, end, None, false, None).unwrap();
        let rule5 = Rule::<serde_json::Value>::new(start, end, None, false, None).unwrap();

        assert!(rule4.has_matching_payload(&rule5).unwrap());
        assert!(!rule4.has_matching_payload(&rule1).unwrap());
    }

    #[test]
    fn test_has_weekdays_in() {
        let start = create_test_datetime(2024, 1, 1, 9, 0, 0);
        let end = create_test_datetime(2024, 1, 1, 17, 0, 0);

        let monday_rule = Rule::<String>::new(start, end, Some(MONDAY), false, None).unwrap();
        let mon_wed_rule =
            Rule::<String>::new(start, end, Some(MONDAY | WEDNESDAY), false, None).unwrap();
        let tue_thu_rule =
            Rule::<String>::new(start, end, Some(TUESDAY | THURSDAY), false, None).unwrap();
        let absolute_rule = Rule::<String>::new(start, end, None, false, None).unwrap();

        assert!(monday_rule.has_weekdays_in(&mon_wed_rule));
        assert!(!monday_rule.has_weekdays_in(&tue_thu_rule));
        assert!(!absolute_rule.has_weekdays_in(&monday_rule));
        assert!(!monday_rule.has_weekdays_in(&absolute_rule));
    }

    #[test]
    fn test_base_rule() {
        let base_rule = Rule::<String>::base_rule();
        assert!(base_rule.off);
        assert!(base_rule.is_absolute());
        assert!(base_rule.payload.is_none());
        assert_eq!(base_rule.start.year(), BASE_RULE_YEAR_START);
        assert_eq!(base_rule.end.year(), BASE_RULE_YEAR_END);
    }
}
