use std::result::Result;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    frame::Frame,
    rule::{relative_to_absolute_rules, Rule},
};

pub struct Availability<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
    Rule<T>: Clone,
{
    pub rules: Vec<Vec<Rule<T>>>,
    pub frames: Vec<Frame<T>>,
}

impl<T> Availability<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
    Rule<T>: Clone,
{
    pub fn new() -> Self {
        Availability {
            rules: vec![vec![Rule::base_rule()]],
            frames: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule<T>, priority: usize) -> Result<(), String> {
        if priority == 0 {
            return Err("Priority 0 is reserved for base rule and cannot be modified".to_string());
        }

        while self.rules.len() <= priority {
            self.rules.push(Vec::new());
        }

        // If there are no existing rules at this priority, we can just add the new rule
        if self.rules[priority].is_empty() {
            self.rules[priority].push(rule);
            return Ok(());
        }

        // Check for overlaps with existing rules
        for existing_rule in self.rules[priority].iter() {
            let overlaps: bool = existing_rule.datetime_overlaps_with(&rule);

            match (overlaps, existing_rule.is_absolute()) {
                // Overlaps + Absolute
                (true, true) => {
                    return Err(format!(
                        "New rule overlaps with existing rule at priority {}. \
                    New rule: {:?} to {:?}, Existing rule: {:?} to {:?}",
                        priority, rule.start, rule.end, existing_rule.start, existing_rule.end
                    )
                    .into())
                }
                // Overlaps + Relative
                (true, false) => {
                    // Only add new rule if none of the weekdays are in existing rule
                    if existing_rule.has_weekdays_in(&rule) {
                        return Err(format!(
                        "New rule overlaps with existing rule at priority {} because of clashing weekdays. \
                        New rule: {:?} to {:?}, Existing rule: {:?} to {:?}",
                        priority, rule.start, rule.end, existing_rule.start, existing_rule.end
                    )
                    .into());
                    }
                }
                // No overlap + Absolute or Relative
                (false, true) | (false, false) => {}
            }
        }

        // If we've made it here, the rule is valid to add
        self.rules[priority].push(rule);
        Ok(())
    }

    /// Remove rule at specific priority and index
    pub fn remove_rule_by_index(
        &mut self,
        priority: usize,
        rule_index: usize,
    ) -> Result<Rule<T>, String> {
        if priority >= self.rules.len() {
            return Err(format!(
                "Priority {} does not exist. Max priority is {}.",
                priority,
                self.rules.len() - 1
            ));
        }

        if priority == 0 {
            return Err("Priority 0 is reserved for base rule and cannot be modified".to_string());
        }

        if rule_index >= self.rules[priority].len() {
            return Err(format!(
                "Rule index {} does not exist at priority level {}.",
                rule_index, priority
            ));
        }

        let removed_rule = self.rules[priority].remove(rule_index);

        // Remove priority level if it is empty and the highest priority level
        if self.rules[priority].is_empty() && self.rules.len() - 1 == priority {
            self.rules.pop();
        }

        Ok(removed_rule)
    }

    pub fn remove_rule_by_datetime(
        &mut self,
        priority: usize,
        datetime: NaiveDateTime,
    ) -> Option<Rule<T>> {
        if priority >= self.rules.len() {
            return None;
        }

        if priority == 0 {
            return None;
        }

        let mut rule_index = None;
        for (i, rule) in self.rules[priority].iter().enumerate() {
            if rule.is_active(datetime) {
                rule_index = Some(i);
                break;
            }
        }

        match rule_index {
            Some(index) => Some(self.rules[priority].remove(index)),
            None => None,
        }
    }

    pub fn remove_rule_by_str(&mut self, priority: usize, datetime: &str) -> Option<Rule<T>> {
        match NaiveDateTime::parse_from_str(datetime, "%Y%m%d%H%M%S") {
            Ok(parsed_datetime) => self.remove_rule_by_datetime(priority, parsed_datetime),
            Err(_) => None,
        }
    }

    /// Construct frames from rules
    pub fn to_frames(&mut self) {
        let mut frames: Vec<Frame<T>> = Vec::new();

        // Process rules from highest to lowest priority
        for priority in (0..self.rules.len()).rev() {
            let mut priority_frames: Vec<Frame<T>> = Vec::new();

            // Convert all rules at this priority level to absolute rules
            let mut absolute_rules: Vec<Rule<T>> = Vec::new();
            for rule in self.rules[priority].iter() {
                if let Ok(abs_rules) = relative_to_absolute_rules(rule.clone()) {
                    absolute_rules.extend(abs_rules);
                }
            }

            // Convert absolute rules to frames
            for rule in absolute_rules {
                let frame = Frame::new(rule.start, rule.end, rule.off, rule.payload.clone());
                priority_frames.push(frame);
            }

            // Merge with existing frames, giving precedence to higher priority frames
            if frames.is_empty() {
                frames = priority_frames;
            } else {
                let mut merged_frames: Vec<Frame<T>> = Vec::new();

                // Sort both frame lists by start time
                frames.sort_by(|a, b| a.start.cmp(&b.start));
                priority_frames.sort_by(|a, b| a.start.cmp(&b.start));

                let mut i = 0;
                let mut j = 0;

                while i < frames.len() || j < priority_frames.len() {
                    if i >= frames.len() {
                        // Add remaining lower priority frames
                        merged_frames.push(priority_frames[j].clone());
                        j += 1;
                        continue;
                    }

                    if j >= priority_frames.len() {
                        // Add remaining higher priority frames
                        merged_frames.push(frames[i].clone());
                        i += 1;
                        continue;
                    }

                    let high_frame = &frames[i];
                    let low_frame = &priority_frames[j];

                    if high_frame.start >= low_frame.end {
                        // No overlap, add lower priority frame
                        merged_frames.push(low_frame.clone());
                        j += 1;
                    } else if low_frame.start >= high_frame.end {
                        // No overlap, add higher priority frame
                        merged_frames.push(high_frame.clone());
                        i += 1;
                    } else {
                        // Overlap exists - keep higher priority frame
                        if low_frame.start < high_frame.start {
                            // Add non-overlapping part of lower priority frame
                            merged_frames.push(Frame::new(
                                low_frame.start,
                                high_frame.start,
                                low_frame.off,
                                low_frame.payload.clone(),
                            ));
                        }

                        merged_frames.push(high_frame.clone());

                        if low_frame.end > high_frame.end {
                            // Add non-overlapping part of lower priority frame
                            merged_frames.push(Frame::new(
                                high_frame.end,
                                low_frame.end,
                                low_frame.off,
                                low_frame.payload.clone(),
                            ));
                        }

                        i += 1;
                        j += 1;
                    }
                }

                frames = merged_frames;
            }
        }

        // Sort final frames by start time
        frames.sort_by(|a, b| a.start.cmp(&b.start));
        self.frames = frames;
    }

    pub fn payload_from_naivedatetime(&self, datetime: NaiveDateTime) -> Option<T> {
        for frame in self.frames.iter() {
            if frame.start <= datetime && frame.end > datetime {
                return frame.payload.clone();
            }
        }
        None
    }

    pub fn payload_from_str(&self, datetime: &str) -> Option<T> {
        match NaiveDateTime::parse_from_str(datetime, "%Y%m%d%H%M%S") {
            Ok(parsed_datetime) => self.payload_from_naivedatetime(parsed_datetime),
            Err(_) => None,
        }
    }

    pub fn is_open_from_naivedatetime(&self, datetime: NaiveDateTime) -> bool {
        for frame in self.frames.iter() {
            if frame.start <= datetime && frame.end > datetime {
                return !frame.off;
            }
        }
        false
    }

    pub fn is_open_from_str(&self, datetime: &str) -> bool {
        match NaiveDateTime::parse_from_str(datetime, "%Y%m%d%H%M%S") {
            Ok(parsed_datetime) => self.is_open_from_naivedatetime(parsed_datetime),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        rulebuilder::RuleBuilder,
        weekdays::{FRIDAY, MONDAY, THURSDAY, TUESDAY, WEDNESDAY},
    };

    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};
    use serde_json::{json, Value};

    fn create_datetime(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, min, sec)
            .unwrap()
    }

    #[test]
    fn test_new_empty() {
        let availability: Availability<Value> = Availability::new();
        assert_eq!(availability.rules.len(), 1); // Should have base rule
        assert_eq!(availability.frames.len(), 0); // No frames yet

        // Check base rule properties
        let base_rule = &availability.rules[0][0];
        assert!(base_rule.off);
        assert!(base_rule.is_absolute());
        assert!(base_rule.payload.is_none());
    }

    #[test]
    fn test_add_rule_priority_validation() {
        let mut availability: Availability<Value> = Availability::new();

        // Try to add rule at priority 0 (should fail)
        let rule = Rule::new(
            create_datetime(2024, 1, 1, 9, 0, 0),
            create_datetime(2024, 1, 1, 17, 0, 0),
            None,
            false,
            None,
        )
        .unwrap();

        let result = availability.add_rule(rule, 0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Priority 0 is reserved for base rule and cannot be modified"
        );
    }

    #[test]
    fn test_add_rule_absolute_overlap() {
        let mut availability: Availability<Value> = Availability::new();

        // Add first rule
        let rule1 = Rule::new(
            create_datetime(2024, 1, 1, 9, 0, 0),
            create_datetime(2024, 1, 1, 17, 0, 0),
            None,
            false,
            None,
        )
        .unwrap();
        availability.add_rule(rule1, 1).unwrap();

        // Try to add overlapping rule at same priority
        let rule2 = Rule::new(
            create_datetime(2024, 1, 1, 12, 0, 0),
            create_datetime(2024, 1, 1, 18, 0, 0),
            None,
            false,
            None,
        )
        .unwrap();

        let result = availability.add_rule(rule2, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("overlaps with existing rule"));
    }

    #[test]
    fn test_add_rule_relative_overlap() {
        let mut availability: Availability<Value> = Availability::new();

        // Add Monday, Tuesday and Wednesday rule
        let rule1 = Rule::new(
            create_datetime(2024, 1, 1, 9, 0, 0),
            create_datetime(2024, 1, 31, 17, 0, 0),
            Some(MONDAY | TUESDAY | WEDNESDAY),
            false,
            None,
        )
        .unwrap();
        availability.add_rule(rule1, 1).unwrap();

        // Try to add overlapping Wednesday, Thursday and Friday rule
        let rule2 = Rule::new(
            create_datetime(2024, 1, 1, 9, 0, 0),
            create_datetime(2024, 1, 31, 17, 0, 0),
            Some(WEDNESDAY | THURSDAY | FRIDAY),
            false,
            None,
        )
        .unwrap();

        let result = availability.add_rule(rule2, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("clashing weekdays"));
    }

    #[test]
    fn test_remove_rule() {
        let mut availability: Availability<Value> = Availability::new();

        let rule = Rule::new(
            create_datetime(2024, 1, 1, 9, 0, 0),
            create_datetime(2024, 1, 1, 17, 0, 0),
            None,
            false,
            None,
        )
        .unwrap();

        availability.add_rule(rule.clone(), 1).unwrap();

        // Remove the rule
        let removed = availability.remove_rule_by_index(1, 0).unwrap();
        assert_eq!(removed.start, rule.start);
        assert_eq!(removed.end, rule.end);

        // Priority level should be removed since it's empty
        assert_eq!(availability.rules.len(), 1); // Only base rule remains

        // Test removing by datetime at index
        availability.add_rule(rule.clone(), 1).unwrap();
        let removed = availability.remove_rule_by_datetime(1, rule.start).unwrap();
        assert_eq!(removed.start, rule.start);

        // Test removing by datetime as string
        availability.add_rule(rule.clone(), 1).unwrap();
        let removed = availability
            .remove_rule_by_str(1, &rule.start.format("%Y%m%d%H%M%S").to_string())
            .unwrap();
        assert_eq!(removed.start, rule.start);

        // Test removing by datetime as str for two overlapping relative rules
        // with different weekdays
        let rule1 = RuleBuilder::new()
            .start_time_str("240101090000")
            .end_time_str("240131170000")
            .monday()
            .tuesday()
            .wednesday()
            .payload(json!({"type": "regular"}))
            .build()
            .unwrap();
        let rule2 = RuleBuilder::new()
            .start_time_str("240101090000")
            .end_time_str("240131170000")
            .thursday()
            .friday()
            .payload(json!({"type": "special"}))
            .build()
            .unwrap();
        availability.add_rule(rule1, 2).unwrap();
        availability.add_rule(rule2, 2).unwrap();
        let removed = availability
            .remove_rule_by_str(2, "20240101120000")
            .unwrap();
        assert_eq!(
            removed.payload.unwrap()["type"].as_str().unwrap(),
            "regular"
        );
        assert_eq!(availability.rules[2].len(), 1);
    }

    #[test]
    fn test_remove_rule_validation() {
        let mut availability: Availability<Value> = Availability::new();

        // Try to remove from non-existent priority
        let result = availability.remove_rule_by_index(1, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Priority 1 does not exist"));

        // Try to remove from priority 0
        let result = availability.remove_rule_by_index(0, 0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Priority 0 is reserved for base rule and cannot be modified"
        );
    }

    #[test]
    fn test_to_frames_priority_ordering() {
        let mut availability: Availability<Value> = Availability::new();

        // Lower priority rule (weekdays)
        let rule1 = Rule::new(
            create_datetime(2024, 1, 1, 9, 0, 0),
            create_datetime(2024, 1, 31, 17, 0, 0),
            Some(MONDAY | TUESDAY | WEDNESDAY | THURSDAY | FRIDAY),
            false,
            Some(json!({"type": "regular"})),
        )
        .unwrap();

        // Higher priority rule (holiday closure)
        let rule2 = Rule::new(
            create_datetime(2024, 1, 15, 0, 0, 0),
            create_datetime(2024, 1, 16, 0, 0, 0),
            None,
            true,
            Some(json!({"type": "holiday"})),
        )
        .unwrap();

        availability.add_rule(rule1, 1).unwrap();
        availability.add_rule(rule2, 2).unwrap();

        availability.to_frames();

        // Find the frame for January 15th
        let holiday_frame = availability
            .frames
            .iter()
            .find(|f| f.start.date() == NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
            .unwrap();

        // Should have holiday payload and be closed
        assert_eq!(
            holiday_frame.payload.as_ref().unwrap()["type"]
                .as_str()
                .unwrap(),
            "holiday"
        );
        assert!(holiday_frame.off);
    }

    #[test]
    fn test_payload_from_datetime() {
        let mut availability: Availability<Value> = Availability::new();

        let rule = Rule::new(
            create_datetime(2024, 1, 1, 9, 0, 0),
            create_datetime(2024, 1, 1, 17, 0, 0),
            None,
            false,
            Some(json!({"shift": "day"})),
        )
        .unwrap();

        availability.add_rule(rule, 1).unwrap();
        availability.to_frames();

        // Check payload during business hours
        let during = create_datetime(2024, 1, 1, 12, 0, 0);
        let payload = availability.payload_from_naivedatetime(during).unwrap();
        assert_eq!(payload["shift"].as_str().unwrap(), "day");

        // Check payload outside business hours
        let before = create_datetime(2024, 1, 1, 8, 0, 0);
        assert!(availability.payload_from_naivedatetime(before).is_none());
    }

    #[test]
    fn test_is_open() {
        let mut availability: Availability<Value> = Availability::new();

        let rule = Rule::new(
            create_datetime(2024, 1, 1, 9, 0, 0),
            create_datetime(2024, 1, 1, 17, 0, 0),
            None,
            false,
            None,
        )
        .unwrap();

        availability.add_rule(rule, 1).unwrap();
        availability.to_frames();

        // Check various times
        assert!(availability.is_open_from_naivedatetime(create_datetime(2024, 1, 1, 12, 0, 0)));
        assert!(!availability.is_open_from_naivedatetime(create_datetime(2024, 1, 1, 8, 0, 0)));
        assert!(!availability.is_open_from_naivedatetime(create_datetime(2024, 1, 1, 17, 0, 0)));
    }

    #[test]
    fn test_string_parsing() {
        let mut availability: Availability<Value> = Availability::new();

        let rule = Rule::new(
            create_datetime(2024, 1, 1, 9, 0, 0),
            create_datetime(2024, 1, 1, 17, 0, 0),
            None,
            false,
            Some(json!({"shift": "day"})),
        )
        .unwrap();

        availability.add_rule(rule, 1).unwrap();
        availability.to_frames();

        // Test valid datetime string
        assert!(availability.is_open_from_str("20240101120000"));
        assert_eq!(
            availability.payload_from_str("20240101120000").unwrap()["shift"]
                .as_str()
                .unwrap(),
            "day"
        );

        // Test invalid datetime string
        assert!(!availability.is_open_from_str("invalid"));
        assert!(availability.payload_from_str("invalid").is_none());
    }
}
