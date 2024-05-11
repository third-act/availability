use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use frame::Frame;
use rule::{Rule, Weekdays};

mod frame;
mod rule;

#[cfg(test)]
mod tests;

/// Rules priority is based on vector index, higher index means higher priority.
/// One frame is a continuous time interval with a state.
/// The state of a frame is determined by the highest priority rule that applies to it.
pub fn get_frames(rules: &Vec<Rule>, start: NaiveDateTime, end: NaiveDateTime) -> Vec<Frame> {
    if rules.is_empty() {
        return vec![Frame {
            start,
            end,
            state: false,
        }];
    }

    let zero_hour: NaiveTime = match NaiveTime::from_hms_opt(0, 0, 0) {
        Some(time) => time,
        None => NaiveTime::default(),
    };

    let mut frames = Vec::new();
    let mut traverse = start;

    while traverse < end {
        let traverse_date = traverse.date();
        let traverse_time = traverse.time();

        for (index, rule) in rules.iter().rev().enumerate() {
            // prio is the index of the rule in the rules vector, since the for loop is reversed we need to reverse the index
            let prio = rules.len() - index - 1;

            // if the traverse date is within the rule's start and end date
            if !(traverse_date >= rule.start_date && traverse_date <= rule.end_date) {
                continue;
            }

            // if the traverse time is within the rules start and end time, if the start and end time is 00:00 any value is valid
            if !((traverse_time >= rule.start_time && traverse_time < rule.end_time)
                || (rule.start_time == zero_hour && rule.end_time == zero_hour))
            {
                continue;
            }

            let end_date_time = match rule.weekdays {
                Some(weekdays) => {
                    if !is_within_weekdays(traverse_date, weekdays) {
                        continue;
                    }

                    //if rule end time is zero_hour, that means that the rule is probably infinite / doesnt have an end time
                    if rule.end_time == zero_hour {
                        traverse_date.and_time(end.time())
                    } else {
                        traverse_date.and_time(rule.end_time)
                    }

                    // traverse_date.and_time(rule.end_time)
                }
                None => end,
            };

            // check if there is a higher priority rule that applies to the frame and return
            // if we should use that rule start as fram end or continue with the current frame end
            // also always use the minimum of the end date and the frame end
            let frame_end = get_frame_end(
                &rules,
                traverse_date,
                end_date_time.date(),
                traverse_time,
                end_date_time.time(),
                prio,
            )
            .min(end);

            frames.push(Frame {
                start: traverse_date.and_time(traverse_time),
                end: frame_end,
                state: rule.state,
            });

            traverse = frame_end;
            break;
        }
    }

    frames
}

fn get_frame_end(
    rules: &Vec<Rule>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    start_time: NaiveTime,
    end_time: NaiveTime,
    lowest_prio: usize,
) -> NaiveDateTime {
    let end_of_day_hour: NaiveTime = match NaiveTime::from_hms_opt(23, 59, 59) {
        Some(time) => time,
        None => NaiveTime::default(),
    };

    let zero_hour: NaiveTime = match NaiveTime::from_hms_opt(0, 0, 0) {
        Some(time) => time,
        None => NaiveTime::default(),
    };

    for rule in rules.iter().skip(lowest_prio).rev() {
        if !(start_date >= rule.start_date && end_date < rule.end_date) {
            continue;
        }

        if !((rule.start_time > start_time && rule.start_time < end_time)
            || (rule.start_time == zero_hour && rule.end_time == zero_hour))
        {
            continue;
        }

        let rule_start = match rule.weekdays {
            Some(weekdays) => {
                if !is_within_weekdays(start_date, weekdays) {
                    continue;
                }

                if rule.start_time == zero_hour {
                    start_date.and_time(end_time)
                } else {
                    start_date.and_time(rule.start_time)
                }

                // start_date.and_time(rule.start_time)
            }
            None => continue,
        };

        return rule_start;
    }

    if end_time == end_of_day_hour {
        let next_date = match end_date.succ_opt() {
            Some(date) => date,
            None => end_date,
        };

        return next_date.and_time(zero_hour);
    }

    return end_date.and_time(end_time);
}

pub fn is_within_weekdays(time: NaiveDate, weekdays: Weekdays) -> bool {
    let weekday = time.weekday();
    let weekday = Weekdays::from_chrono_weekday(weekday);
    weekdays.intersects(weekday)
}
