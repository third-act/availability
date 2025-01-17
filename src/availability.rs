use std::{fmt, result::Result};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    frame::Frame,
    rule::{relative_to_absolute_rules, Rule},
};

/// Represents the availability schedule with priority-based rules.
///
/// The `Availability` struct manages a collection of rules that define availability
/// over time. Rules can overlap in dates if weekdays don't overlap.
/// Rules are prioritized, where a higher priority values takes precedence over rules with
/// lower priority values. A continuous vector of frames is generated from the rules.
///
/// Start and end for rules and frames are always inclusive and exclusive respectively.
///
/// Note that the base rule is always present at priority 0, and it is always "off" (closed).
/// You cannot add, remove, or modify the base rule.
///
/// # Type Parameters
///
/// - `T`: The type of the payload attached to each frame. Must implement `Serialize`, `Deserialize`,
///   and `Clone`.
///
#[derive(Default)]
pub struct Availability<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
    Rule<T>: Clone,
{
    pub rules: Vec<Vec<Rule<T>>>,
    pub(crate) frames: Vec<Frame<T>>,
}

impl<T> fmt::Display for Availability<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
    Rule<T>: fmt::Display,
    Frame<T>: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Availability Frames:")?;
        for frame in &self.frames {
            writeln!(f, "  {}", frame)?;
        }
        Ok(())
    }
}

impl<T> Availability<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
    Rule<T>: Clone,
{
    /// Creates a new, empty `Availability` instance.
    ///
    /// Initializes an `Availability` with no rules. The base rule is automatically included
    /// with the lowest priority to cover all possible date-times as "off" (closed).
    pub fn new() -> Self {
        Availability {
            rules: vec![vec![Rule::base_rule()]],
            frames: Vec::new(),
        }
    }

    /// Adds a new rule with the specified priority.
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
                    ))
                }
                // Overlaps + Relative
                (true, false) => {
                    // Only add new rule if none of the weekdays are in existing rule
                    if existing_rule.has_weekdays_in(&rule) {
                        return Err(format!(
                        "New rule overlaps with existing rule at priority {} because of clashing weekdays. \
                        New rule: {:?} to {:?}, Existing rule: {:?} to {:?}",
                        priority, rule.start, rule.end, existing_rule.start, existing_rule.end
                    ));
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

    /// Retrieves all generated frames
    pub fn get_frames(&self) -> &Vec<Frame<T>> {
        &self.frames
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
        match NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S") {
            Ok(parsed_datetime) => self.remove_rule_by_datetime(priority, parsed_datetime),
            Err(_) => None,
        }
    }

    /// Converts all added rules into a sequence of non-overlapping, time-sorted frames within the specified range.
    ///
    /// This method processes the rules based on their priorities, resolving overlaps by giving precedence
    /// to higher priority rules. The resulting frames represent distinct time intervals with their corresponding
    /// availability status and payload.
    ///
    /// # Parameters
    ///
    /// - `start`: The start datetime of the range to generate frames for. Start is inclusive.
    /// - `end`: The end datetime of the range to generate frames for. End is exclusive.
    pub fn to_frames_in_range(&mut self, start: NaiveDateTime, end: NaiveDateTime) {
        let mut frames: Vec<Frame<T>> = Vec::new();

        // Process rules from highest to lowest priority
        for priority in (1..self.rules.len()).rev() {
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
                if rule.end <= start || rule.start >= end {
                    continue;
                }
                // Rule is completely within range
                if rule.start >= start && rule.end < end {
                    let frame = Frame::new(rule.start, rule.end, rule.off, rule.payload.clone());
                    priority_frames.push(frame);
                }
                // Rule starts before range but ends within range
                else if rule.start < start && rule.end < end {
                    let frame = Frame::new(start, rule.end, rule.off, rule.payload.clone());
                    priority_frames.push(frame);
                }
                // Rule starts within range but ends after range
                else if rule.start >= start && rule.start < end && rule.end >= end {
                    let frame = Frame::new(rule.start, end, rule.off, rule.payload.clone());
                    priority_frames.push(frame);
                }
                // Rule starts before range and ends after range
                else if rule.start < start && rule.end >= end {
                    let frame = Frame::new(start, end, rule.off, rule.payload.clone());
                    priority_frames.push(frame);
                }
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
                            let frame = Frame::new(
                                low_frame.start,
                                high_frame.start,
                                low_frame.off,
                                low_frame.payload.clone(),
                            );
                            merged_frames.push(frame);
                        }
                        merged_frames.push(high_frame.clone());
                        if low_frame.end > high_frame.end {
                            // Add non-overlapping part of lower priority frame
                            let frame = Frame::new(
                                high_frame.end,
                                low_frame.end,
                                low_frame.off,
                                low_frame.payload.clone(),
                            );
                            merged_frames.push(frame);
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

        // Insert base rule if first frame from custom rules is not at start
        if !frames.is_empty() && frames[0].start > start {
            frames.insert(0, Frame::new(start, frames[0].start, true, None));
        }

        // Fill gaps in custom rules with base rule
        let mut i = 0;
        while i + 1 < frames.len() {
            let this_end = frames[i].end;
            let next_start = frames[i + 1].start;
            if this_end < next_start {
                // Gap from [this_end, next_start)
                frames.insert(i + 1, Frame::new(this_end, next_start, true, None));
            }
            i += 1;
        }

        // Fill trailing gap with base rule
        if let Some(last_frame) = frames.last() {
            if last_frame.end < end {
                let frame: Frame<T> = Frame::new(last_frame.end, end, true, None);
                frames.push(frame);
            }
        }
        // If no frames at all were built, create one that covers [start, end) with the base rule
        if frames.is_empty() {
            frames.push(Frame::new(start, end, true, None));
        }
        frames.retain(|f| f.duration().num_seconds() > 0);
        self.frames = frames;
    }

    /// Converts all added rules into frames within the specified range using datetime strings.
    ///
    /// This is a convenience method that parses the provided datetime strings and calls
    /// `to_frames_in_range`.
    ///
    /// The datetime strings must be in the `"YYYY-MM-DD HH:MM:SS"` format.
    ///
    /// # Parameters
    ///
    /// - `start_str`: A string slice representing the start datetime in `"YYYY-MM-DD HH:MM:SS"` format. Start is inclusive.
    /// - `end_str`: A string slice representing the end datetime in `"YYYY-MM-DD HH:MM:SS"` format. End is exclusive.
    pub fn to_frames_in_range_str(&mut self, start: &str, end: &str) {
        if let (Ok(parsed_start), Ok(parsed_end)) = (
            NaiveDateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S"),
            NaiveDateTime::parse_from_str(end, "%Y-%m-%d %H:%M:%S"),
        ) {
            self.to_frames_in_range(parsed_start, parsed_end)
        }
    }

    pub fn get_frame(&self, datetime: NaiveDateTime) -> Option<Frame<T>> {
        let mut current_frame: Option<Frame<T>> = None;
        for frame in self.frames.iter() {
            if frame.start <= datetime && frame.end > datetime {
                current_frame = Some(frame.clone());
            }
        }
        current_frame
    }

    /// Retrieves the frame corresponding to the specified datetime string.
    /// The datetime string must be in the `"YYYY-MM-DD HH:MM:SS"` format.
    pub fn get_frame_from_str(&self, datetime: &str) -> Option<Frame<T>> {
        match NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S") {
            Ok(parsed_datetime) => self.get_frame(parsed_datetime),
            Err(_) => None,
        }
    }

    /// Retrieves all generated frames.
    pub fn frames(&self) -> &Vec<Frame<T>> {
        &self.frames
    }

    /// Clears all generated frames.
    pub fn clear_frames(&mut self) {
        self.frames.clear();
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
            .remove_rule_by_str(1, &rule.start.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap();
        assert_eq!(removed.start, rule.start);

        // Test removing by datetime as str for two overlapping relative rules
        // with different weekdays
        let rule1 = RuleBuilder::new()
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-31 17:00:00")
            .monday()
            .tuesday()
            .wednesday()
            .payload(json!({"type": "regular"}))
            .build()
            .unwrap();
        let rule2 = RuleBuilder::new()
            .start_time_str("2024-01-01 09:00:00")
            .end_time_str("2024-01-31 17:00:00")
            .thursday()
            .friday()
            .payload(json!({"type": "special"}))
            .build()
            .unwrap();
        availability.add_rule(rule1, 2).unwrap();
        availability.add_rule(rule2, 2).unwrap();
        let removed = availability
            .remove_rule_by_str(2, "2024-01-01 12:00:00")
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
    fn test_to_frames_in_range() {
        use serde_json::json;

        // 1) No custom rules: everything should be base-rule "off"
        {
            let mut availability: Availability<Value> = Availability::new();
            let start = create_datetime(2024, 1, 1, 0, 0, 0);
            let end = create_datetime(2024, 1, 1, 23, 59, 0);

            availability.to_frames_in_range(start, end);
            // Expect exactly 1 frame from [start..end], off = true
            assert_eq!(
                availability.frames.len(),
                1,
                "Should have exactly one frame for entire range"
            );
            let frame = &availability.frames[0];
            assert_eq!(frame.start, start);
            assert_eq!(frame.end, end);
            assert!(frame.off);
        }

        // 2) Single custom rule covers a subset of the day, should fill gaps with base rule
        {
            let mut availability: Availability<Value> = Availability::new();
            // Add a single open rule from 09:00 to 12:00
            let rule = Rule::new(
                create_datetime(2024, 1, 1, 9, 0, 0),
                create_datetime(2024, 1, 1, 12, 0, 0),
                None,  // No weekdays => absolute
                false, // off=false => "open"
                Some(json!({"info": "morning shift"})),
            )
            .unwrap();
            availability.add_rule(rule, 1).unwrap();

            let start = create_datetime(2024, 1, 1, 8, 0, 0);
            let end = create_datetime(2024, 1, 1, 13, 0, 0);

            availability.to_frames_in_range(start, end);
            // Expect frames:
            //   1) [08:00..09:00] off (base rule)
            //   2) [09:00..12:00] on (custom rule)
            //   3) [12:00..13:00] off (base rule)
            assert_eq!(availability.frames.len(), 3);
            assert_eq!(
                availability.frames[0].start,
                create_datetime(2024, 1, 1, 8, 0, 0)
            );
            assert_eq!(
                availability.frames[0].end,
                create_datetime(2024, 1, 1, 9, 0, 0)
            );
            assert!(availability.frames[0].off);

            assert_eq!(
                availability.frames[1].start,
                create_datetime(2024, 1, 1, 9, 0, 0)
            );
            assert_eq!(
                availability.frames[1].end,
                create_datetime(2024, 1, 1, 12, 0, 0)
            );
            assert!(!availability.frames[1].off);
            assert_eq!(
                availability.frames[1].payload.as_ref().unwrap()["info"],
                "morning shift"
            );

            assert_eq!(
                availability.frames[2].start,
                create_datetime(2024, 1, 1, 12, 0, 0)
            );
            assert_eq!(
                availability.frames[2].end,
                create_datetime(2024, 1, 1, 13, 0, 0)
            );
            assert!(availability.frames[2].off);
        }

        // 3) Two overlapping rules:
        //    - Lower priority “open” all day
        //    - Higher priority “closed” from 10:00 to 11:00
        //    => result should have open from 09:00..10:00, closed 10:00..11:00, open 11:00..12:00
        {
            let mut availability: Availability<Value> = Availability::new();

            let all_day_rule = Rule::new(
                create_datetime(2024, 1, 1, 9, 0, 0),
                create_datetime(2024, 1, 1, 12, 0, 0),
                None,
                false,
                Some(json!({"info": "low-prio open"})),
            )
            .unwrap();
            // This is lower priority => 1
            availability.add_rule(all_day_rule, 1).unwrap();

            // Higher priority => 2
            let closed_mid_rule = Rule::new(
                create_datetime(2024, 1, 1, 10, 0, 0),
                create_datetime(2024, 1, 1, 11, 0, 0),
                None,
                true, // closed
                Some(json!({"info": "high-prio closed"})),
            )
            .unwrap();
            availability.add_rule(closed_mid_rule, 2).unwrap();

            availability.to_frames_in_range(
                create_datetime(2024, 1, 1, 9, 0, 0),
                create_datetime(2024, 1, 1, 12, 0, 0),
            );
            // Expect frames:
            //  [09:00..10:00] open (low-prio)
            //  [10:00..11:00] closed (high-prio overrides)
            //  [11:00..12:00] open (low-prio)
            assert_eq!(availability.frames.len(), 3);

            let f1 = &availability.frames[0];
            assert_eq!(f1.start, create_datetime(2024, 1, 1, 9, 0, 0));
            assert_eq!(f1.end, create_datetime(2024, 1, 1, 10, 0, 0));
            assert!(!f1.off);
            assert_eq!(f1.payload.as_ref().unwrap()["info"], "low-prio open");

            let f2 = &availability.frames[1];
            assert_eq!(f2.start, create_datetime(2024, 1, 1, 10, 0, 0));
            assert_eq!(f2.end, create_datetime(2024, 1, 1, 11, 0, 0));
            assert!(f2.off);
            assert_eq!(f2.payload.as_ref().unwrap()["info"], "high-prio closed");

            let f3 = &availability.frames[2];
            assert_eq!(f3.start, create_datetime(2024, 1, 1, 11, 0, 0));
            assert_eq!(f3.end, create_datetime(2024, 1, 1, 12, 0, 0));
            assert!(!f3.off);
            assert_eq!(f3.payload.as_ref().unwrap()["info"], "low-prio open");
        }

        // 4) **Partial coverage**:
        //    - One rule from 00:00..06:00 open
        //    - Another rule from 06:00..12:00 closed
        //    - Range asked: 00:00..12:00 => Should see exactly those two frames, no gap
        {
            let mut availability: Availability<Value> = Availability::new();
            let open_rule = Rule::new(
                create_datetime(2024, 1, 1, 0, 0, 0),
                create_datetime(2024, 1, 1, 6, 0, 0),
                None,
                false, // open
                None,
            )
            .unwrap();
            let closed_rule = Rule::new(
                create_datetime(2024, 1, 1, 6, 0, 0),
                create_datetime(2024, 1, 1, 12, 0, 0),
                None,
                true, // closed
                None,
            )
            .unwrap();
            // Both are same priority => no overlap, no override
            availability.add_rule(open_rule, 1).unwrap();
            availability.add_rule(closed_rule, 1).unwrap();

            let start = create_datetime(2024, 1, 1, 0, 0, 0);
            let end = create_datetime(2024, 1, 1, 12, 0, 0);
            availability.to_frames_in_range(start, end);

            // Expect 2 frames exactly, no base-rule gap
            //   [00:00..06:00] off=false
            //   [06:00..12:00] off=true
            assert_eq!(availability.frames.len(), 2);
            assert_eq!(
                availability.frames[0].start,
                create_datetime(2024, 1, 1, 0, 0, 0)
            );
            assert_eq!(
                availability.frames[0].end,
                create_datetime(2024, 1, 1, 6, 0, 0)
            );
            assert!(!availability.frames[0].off);

            assert_eq!(
                availability.frames[1].start,
                create_datetime(2024, 1, 1, 6, 0, 0)
            );
            assert_eq!(
                availability.frames[1].end,
                create_datetime(2024, 1, 1, 12, 0, 0)
            );
            assert!(availability.frames[1].off);
        }
    }

    #[test]
    fn crash_test() {
        let mut availability: Availability<()> = Availability::new();
        let rule: Rule<()> = RuleBuilder::new()
            .start_time_str("1970-01-01 00:00:00")
            .end_time_str("2999-12-30 01:00:00")
            .all_weekdays()
            .off(false)
            .build()
            .unwrap();
        let _ = availability.add_rule(rule, 1);
        availability.to_frames_in_range_str("2024-10-29 13:20:27", "2024-11-01 09:20:00");
        let frames = availability.get_frames();
        assert_eq!(frames.len(), 7);
    }

    #[test]
    fn crash_test2() {
        let mut availability: Availability<()> = Availability::new();
        let rule: Rule<()> = RuleBuilder::new()
            .start_time_str("1970-01-01 00:00:00")
            .end_time_str("2999-12-30 00:00:00")
            .all_weekdays()
            .off(false)
            .build()
            .unwrap();
        let _ = availability.add_rule(rule, 1);
        availability.to_frames_in_range_str("2024-10-29 13:20:27", "2024-11-01 09:20:00");
        let frames = availability.get_frames();
        assert_eq!(frames.len(), 1);
    }

    #[test]
    fn test_is_all_weekdays_relative_absolute() {
        let rule: Rule<()> = RuleBuilder::new()
            .start_time_str("2024-01-01 00:00:00")
            .end_time_str("2025-01-01 00:00:00")
            .monday()
            .friday()
            .build()
            .unwrap();
        assert_eq!(rule.is_relative(), true);
        let rule: Rule<()> = RuleBuilder::new()
            .start_time_str("2024-01-01 00:00:00")
            .end_time_str("2024-01-31 00:00:00")
            .all_weekdays()
            .build()
            .unwrap();
        assert_eq!(rule.is_absolute(), true);
    }

    #[test]
    fn test_midnight_to_midnight() {
        let mut availability: Availability<()> = Availability::new();
        let rule_absolute: Rule<()> = RuleBuilder::new()
            .start_time_str("2024-01-02 00:00:00")
            .end_time_str("2024-01-03 00:00:00")
            .all_weekdays()
            .build()
            .unwrap();
        assert!(rule_absolute.is_absolute()); // All weekdays and midnight to midnight should be absolute
        availability.add_rule(rule_absolute, 1).unwrap();
        availability.to_frames_in_range_str("2024-01-01 00:00:00", "2024-01-04 00:00:00");
        let frames = availability.get_frames();
        assert_eq!(frames.len(), 3);
        let rule_relative: Rule<()> = RuleBuilder::new()
            .start_time_str("2024-01-02 00:00:00")
            .end_time_str("2024-01-03 23:59:60")
            .all_weekdays()
            .build()
            .unwrap();
        assert!(rule_relative.is_relative()); // All weekdays and midnight to 1 second before midnight should be relative
        availability = Availability::new();
        availability.add_rule(rule_relative, 1).unwrap();
        availability.to_frames_in_range_str("2024-01-01 00:00:00", "2024-01-04 00:00:00");
        // 2024-01-01 00:00:00 to 2024-01-02 00:00:00 off
        // 2024-01-02 00:00:00 to 2024-01-02 23:59:60 on
        // 2024-01-03 00:00:00 to 2024-01-03 23:59:60 on
        let frames = availability.get_frames();
        for frame in frames.iter() {
            println!("{}", frame);
        }
        assert_eq!(frames.len(), 3);

        // With payload
        let rule_relative: Rule<Value> = RuleBuilder::new()
            .start_time_str("2024-01-02 00:00:00")
            .end_time_str("2024-01-03 23:59:60")
            .all_weekdays()
            .payload(json!({"type": "regular"}))
            .build()
            .unwrap();
        assert!(rule_relative.is_relative()); // All weekdays and midnight to 1 second before midnight should be relaitve
        let mut availability: Availability<Value> = Availability::new();
        availability.add_rule(rule_relative, 1).unwrap();
        availability.to_frames_in_range_str("2024-01-01 00:00:00", "2024-01-04 00:00:00");
        let frames = availability.get_frames();
        for frame in frames.iter() {
            println!("{}", frame);
        }
        assert_eq!(frames.len(), 3);
    }

    #[test]
    fn test_frame_duration_midnight() {
        // Rule from 00:00:00 to 23:59:60 on same day (leap second) gives duration 24:00:00 (1 full day)
        let mut availability: Availability<()> = Availability::new();
        let rule: Rule<()> = RuleBuilder::new()
            .start_time_str("2024-01-01 00:00:00")
            .end_time_str("2024-01-01 23:59:60")
            .off(false)
            .all_weekdays()
            .build()
            .unwrap();
        availability.add_rule(rule, 1).unwrap();
        availability.to_frames_in_range_str("2024-01-01 00:00:00", "2024-01-02 00:00:00");
        let frames = availability.get_frames();
        assert_eq!(frames.len(), 1);
        assert!(frames[0].duration().num_seconds() == 86400);
        // Rule from 00:00:00 to 00:00:00 next day gives duration 24:00:00 (1 full day)
        let mut availability: Availability<()> = Availability::new();
        let rule: Rule<()> = RuleBuilder::new()
            .start_time_str("2024-01-01 00:00:00")
            .end_time_str("2024-01-02 00:00:00")
            .off(false)
            .all_weekdays()
            .build()
            .unwrap();
        availability.add_rule(rule, 1).unwrap();
        availability.to_frames_in_range_str("2024-01-01 00:00:00", "2024-01-02 00:00:00");
        let frames = availability.get_frames();
        for frame in frames.iter() {
            println!("{}", frame);
        }
        assert_eq!(frames.len(), 1);
        assert!(frames[0].duration().num_seconds() == 86400);
    }
}
