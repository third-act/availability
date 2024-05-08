use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use frame::Frame;
use rule::{Rule, Weekdays};

mod frame;
mod rule;

#[cfg(test)]
mod tests;

/// Rules priority is based on vector index, higher index means higher priority.
/// One frame is a continuous time interval with a state.
/// The state of a frame is determined by the highest priority rule that applies to it.
pub fn get_frames(rules: Vec<Rule>, start: NaiveDateTime, end: NaiveDateTime) -> Vec<Frame> {
    if rules.is_empty() {
        return vec![Frame {
            start,
            end,
            state: false,
        }];
    }

    let mut frames = Vec::new();
    let mut traverse = start;

    while traverse < end {
        let traverse_date = traverse.date();
        let traverse_time = traverse.time();

        for (index, rule) in rules.iter().rev().enumerate() {
            // prio is the index of the rule in the rules vector
            let prio = rules.len() - index - 1;

            // if the traverse date is within the rule's start and end date
            if !(traverse_date >= rule.start_date && traverse_date < rule.end_date) {
                println!("SKIP TRAVERSE: {}", traverse_time);
                println!("SKIP: {} - {}", rule.start_date, rule.end_date);
                continue;
            }

            // if the traverse time is within the rules start and end time, if teh start and end time is 00:00 any value is valid
            if !((traverse_time >= rule.start_time && traverse_time < rule.end_time)
                || (rule.start_time == NaiveTime::from_hms(0, 0, 0)
                    && rule.end_time == NaiveTime::from_hms(0, 0, 0)))
            {
                println!("SKIP TRAVERSE TIME: {}", traverse_time);
                println!("SKIP: {} - {}", rule.start_time, rule.end_time);
                continue;
            }

            let end_date_time = match rule.weekdays {
                Some(weekdays) => {
                    if !is_within_weekdays(traverse_date, weekdays) {
                        continue;
                    }

                    let rule_end_time = traverse_date.and_time(rule.end_time);

                    rule_end_time
                }
                None => end,
            };

            println!("RULE: {} - {}", rule.start_date, rule.end_date);
            println!(
                "TRAVERSE & END: {} - {}",
                traverse_date.and_time(traverse_time),
                end_date_time
            );

            let frame_end = get_frame_end(
                &rules,
                traverse_date,
                end_date_time.date(),
                traverse_time,
                end_date_time.time(),
                prio,
            )
            .min(end);

            println!(
                "FRAME: {} - {}",
                traverse_date.and_time(traverse_time),
                frame_end
            );

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
    println!("LOWEST PRIO: {}", lowest_prio);
    for rule in rules.iter().skip(lowest_prio).rev() {
        if !(start_date >= rule.start_date && end_date < rule.end_date) {
            println!(
                "SKIPPING RULE FRAME: {} - {}",
                rule.start_date, rule.end_date
            );
            continue;
        }

        if !((rule.start_time >= start_time && rule.end_time < end_time)
            || (rule.start_time == NaiveTime::from_hms(0, 0, 0)
                && rule.end_time == NaiveTime::from_hms(0, 0, 0)))
        {
            println!("START TIME: {}", start_time);
            println!("END TIME: {}", end_time);
            println!(
                "SKIPPING RULE FRAME TIME: {} - {}",
                rule.start_time, rule.end_time
            );

            continue;
        }

        let rule_start = match rule.weekdays {
            Some(weekdays) => {
                if !is_within_weekdays(start_date, weekdays) {
                    println!("SKIPPING RULE FRAME WEEKDAYS: {:?}", weekdays);
                    continue;
                }

                let rule_start_time = start_date.and_time(rule.start_time);

                rule_start_time
            }
            None => continue,
        };

        return rule_start;
    }

    return end_date.and_time(end_time);
}

// pub fn is_within_rule_time(time: NaiveDateTime, rule: &Rule) -> bool {
//     let time_of_day = time.time();
//     let start_of_day = rule.start.time();
//     let mut end_of_day = rule.end.time();

//     if end_of_day == NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default() {
//         end_of_day = NaiveTime::from_hms_opt(23, 59, 59).unwrap_or_default();
//     }

//     // Check if the time falls within the interval defined by start and end
//     time_of_day >= start_of_day && time_of_day < end_of_day
// }

pub fn is_within_weekdays(time: NaiveDate, weekdays: Weekdays) -> bool {
    let weekday = time.weekday();
    let weekday = Weekdays::from_chrono_weekday(weekday);
    weekdays.intersects(weekday)
}
