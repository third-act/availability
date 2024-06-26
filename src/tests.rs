use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::{
    frame::Frame,
    get_frames,
    rule::{Rule, Weekdays},
};

//////////////////////////////////////////
// Test with println! to see the output //
// cargo test -- --nocapture            //
//////////////////////////////////////////

#[test]
fn get_frames_two_rules_regular_opening_times() {
    use chrono::NaiveDateTime;

    let weekdays = Weekdays::MONDAY
        | Weekdays::TUESDAY
        | Weekdays::WEDNESDAY
        | Weekdays::THURSDAY
        | Weekdays::FRIDAY;

    let rules = vec![
        Rule::<()> {
            start_date: NaiveDate::from_str("2000-01-01").unwrap(),
            end_date: NaiveDate::from_str("3000-01-01").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("00:00:00").unwrap(),
            weekdays: None,
            state: false,
            payload: None,
        },
        Rule::<()> {
            start_date: NaiveDate::from_str("2024-01-01").unwrap(),
            end_date: NaiveDate::from_str("2025-01-01").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays),
            state: true,
            payload: None,
        },
        Rule::<()> {
            start_date: NaiveDate::from_str("2024-06-03").unwrap(),
            end_date: NaiveDate::from_str("2024-06-06").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays),
            state: false,
            payload: None,
        },
    ];

    let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 1);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end,
            state: true,
            payload: None,
        }
    );
}

// ADD MORE TEST
// 1. Test for the case when there is no rule
// 2. Test for the case when there is only one rule
// 3. Test for the case when there are multiple rules (>2)

#[test]
fn get_frames_no_rule() {
    use chrono::NaiveDateTime;

    let rules = vec![];

    let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 1);
    assert_eq!(
        frames[0],
        Frame::<()> {
            start,
            end,
            state: false,
            payload: None,
        }
    );
}

#[test]
fn get_frames_one_rule_with_baseline() {
    use chrono::NaiveDateTime;

    let weekdays = Weekdays::MONDAY
        | Weekdays::TUESDAY
        | Weekdays::WEDNESDAY
        | Weekdays::THURSDAY
        | Weekdays::FRIDAY;

    let rules = vec![
        Rule::<()> {
            start_date: NaiveDate::from_str("2000-01-01").unwrap(),
            end_date: NaiveDate::from_str("3000-01-01").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("00:00:00").unwrap(),
            weekdays: None,
            state: false,
            payload: None,
        },
        Rule::<()> {
            start_date: NaiveDate::from_str("2024-01-01").unwrap(),
            end_date: NaiveDate::from_str("2025-01-01").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays),
            state: true,
            payload: None,
        },
    ];

    let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 1);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end,
            state: true,
            payload: None,
        }
    );
}

#[test]
fn get_frames_always_open() {
    let all_weekdays = Weekdays::MONDAY
        | Weekdays::TUESDAY
        | Weekdays::WEDNESDAY
        | Weekdays::THURSDAY
        | Weekdays::FRIDAY
        | Weekdays::SATURDAY
        | Weekdays::SUNDAY;

    let rules = vec![Rule::<()> {
        start_date: NaiveDate::from_str("2000-01-01").unwrap(),
        end_date: NaiveDate::from_str("3000-01-01").unwrap(),
        start_time: NaiveTime::from_str("00:00:00").unwrap(),
        end_time: NaiveTime::from_str("00:00:00").unwrap(),
        weekdays: Some(all_weekdays),
        state: true,
        payload: None,
    }];

    let start = NaiveDateTime::from_str("2024-04-29T08:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-04-29T22:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 1);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end,
            state: true,
            payload: None,
        }
    );
}

#[test]
fn get_frames_multiple_rules_scenario_1() {
    use chrono::NaiveDateTime;

    let weekdays_1 = Weekdays::MONDAY
        | Weekdays::TUESDAY
        | Weekdays::WEDNESDAY
        | Weekdays::THURSDAY
        | Weekdays::FRIDAY;

    let weekdays_2 = Weekdays::SATURDAY | Weekdays::SUNDAY;

    let weekdays_3 = Weekdays::MONDAY | Weekdays::TUESDAY;

    let rules = vec![
        Rule {
            start_date: NaiveDate::from_str("2000-01-01").unwrap(),
            end_date: NaiveDate::from_str("3000-01-01").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("00:00:00").unwrap(),
            weekdays: None,
            state: false,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-01-01").unwrap(),
            end_date: NaiveDate::from_str("2025-01-01").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays_1),
            state: true,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-06-03").unwrap(),
            end_date: NaiveDate::from_str("2024-06-06").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays_3),
            state: false,
            payload: None,
        },
        Rule::<()> {
            start_date: NaiveDate::from_str("2024-05-11").unwrap(),
            end_date: NaiveDate::from_str("2025-05-11").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("23:59:59").unwrap(),
            weekdays: Some(weekdays_2),
            state: true,
            payload: None,
        },
    ];

    let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 1);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end,
            state: true,
            payload: None,
        }
    );

    let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-04-29T18:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 2);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end: NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap(),
            state: true,
            payload: None,
        }
    );
    assert_eq!(
        frames[1],
        Frame {
            start: NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap(),
            end,
            state: false,
            payload: None,
        }
    );

    let start = NaiveDateTime::from_str("2024-06-04T12:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-06-04T16:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 1);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end,
            state: false,
            payload: None,
        }
    );

    let start = NaiveDateTime::from_str("2024-05-07T03:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-05-07T20:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    // assert_eq!(frames.len(), 3);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end: NaiveDateTime::from_str("2024-05-07T08:00:00").unwrap(),
            state: false,
            payload: None,
        }
    );
    assert_eq!(
        frames[1],
        Frame {
            start: NaiveDateTime::from_str("2024-05-07T08:00:00").unwrap(),
            end: NaiveDateTime::from_str("2024-05-07T16:00:00").unwrap(),
            state: true,
            payload: None,
        }
    );
    assert_eq!(
        frames[2],
        Frame {
            start: NaiveDateTime::from_str("2024-05-07T16:00:00").unwrap(),
            end,
            state: false,
            payload: None,
        }
    );
}

#[test]
fn get_frames_multiple_rules_scenario_2() {
    use chrono::NaiveDateTime;

    let weekdays_1 = Weekdays::MONDAY
        | Weekdays::TUESDAY
        | Weekdays::WEDNESDAY
        | Weekdays::THURSDAY
        | Weekdays::FRIDAY;

    let weekdays_2 = Weekdays::SATURDAY | Weekdays::SUNDAY;

    let weekdays_3 = Weekdays::MONDAY | Weekdays::TUESDAY;

    let rules: Vec<Rule<()>> = vec![
        Rule {
            start_date: NaiveDate::from_str("2000-01-01").unwrap(),
            end_date: NaiveDate::from_str("3000-01-01").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("00:00:00").unwrap(),
            weekdays: None,
            state: false,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-01-01").unwrap(),
            end_date: NaiveDate::from_str("2025-01-01").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays_1),
            state: true,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-06-03").unwrap(),
            end_date: NaiveDate::from_str("2024-06-06").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays_3),
            state: false,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-05-11").unwrap(),
            end_date: NaiveDate::from_str("2025-05-11").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("23:59:59").unwrap(),
            weekdays: Some(weekdays_2),
            state: true,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-05-12").unwrap(),
            end_date: NaiveDate::from_str("2025-05-12").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("03:00:00").unwrap(),
            weekdays: Some(weekdays_2),
            state: true,
            payload: None,
        },
    ];

    let start = NaiveDateTime::from_str("2024-05-11T11:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-05-12T04:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 3);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end: NaiveDateTime::from_str("2024-05-12T00:00:00").unwrap(),
            state: true,
            payload: None,
        }
    );
    assert_eq!(
        frames[1],
        Frame {
            start: NaiveDateTime::from_str("2024-05-12T00:00:00").unwrap(),
            end: NaiveDateTime::from_str("2024-05-12T03:00:00").unwrap(),
            state: true,
            payload: None,
        }
    );
    assert_eq!(
        frames[2],
        Frame {
            start: NaiveDateTime::from_str("2024-05-12T03:00:00").unwrap(),
            end,
            state: false,
            payload: None,
        }
    );
}

#[test]
fn get_frames_multiple_rules_scenario_3() {
    let weekdays_1 = Weekdays::MONDAY
        | Weekdays::TUESDAY
        | Weekdays::WEDNESDAY
        | Weekdays::THURSDAY
        | Weekdays::FRIDAY;

    let rules: Vec<Rule<()>> = vec![
        Rule {
            start_date: NaiveDate::from_str("2000-01-01").unwrap(),
            end_date: NaiveDate::from_str("3000-01-01").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("00:00:00").unwrap(),
            weekdays: None,
            state: false,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-01-01").unwrap(),
            end_date: NaiveDate::from_str("2025-01-01").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays_1),
            state: true,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-01-01").unwrap(),
            end_date: NaiveDate::from_str("2025-01-01").unwrap(),
            start_time: NaiveTime::from_str("20:00:00").unwrap(),
            end_time: NaiveTime::from_str("23:59:59").unwrap(),
            weekdays: Some(weekdays_1),
            state: true,
            payload: None,
        },
    ];

    let start = NaiveDateTime::from_str("2024-05-07T08:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-05-07T22:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 3);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end: NaiveDateTime::from_str("2024-05-07T16:00:00").unwrap(),
            state: true,
            payload: None,
        }
    );
    assert_eq!(
        frames[1],
        Frame {
            start: NaiveDateTime::from_str("2024-05-07T16:00:00").unwrap(),
            end: NaiveDateTime::from_str("2024-05-07T20:00:00").unwrap(),
            state: false,
            payload: None,
        }
    );
    assert_eq!(
        frames[2],
        Frame {
            start: NaiveDateTime::from_str("2024-05-07T20:00:00").unwrap(),
            end,
            state: true,
            payload: None,
        }
    );
}

#[test]
fn get_frames_multiple_rules_scenario_4() {
    let weekdays_1 = Weekdays::MONDAY
        | Weekdays::TUESDAY
        | Weekdays::WEDNESDAY
        | Weekdays::THURSDAY
        | Weekdays::FRIDAY;

    let rules: Vec<Rule<()>> = vec![
        Rule {
            start_date: NaiveDate::from_str("2000-01-01").unwrap(),
            end_date: NaiveDate::from_str("3000-01-01").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("00:00:00").unwrap(),
            weekdays: None,
            state: false,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-01-01").unwrap(),
            end_date: NaiveDate::from_str("2025-01-01").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays_1),
            state: true,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-12-24").unwrap(),
            end_date: NaiveDate::from_str("2024-12-24").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("23:59:59").unwrap(),
            weekdays: Some(weekdays_1),
            state: false,
            payload: None,
        },
    ];

    let start = NaiveDateTime::from_str("2024-12-24T12:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-12-24T16:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 1);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end,
            state: false,
            payload: None,
        }
    );
}

#[test]
fn get_frames_exact_time_frame() {
    let weekdays_1 = Weekdays::MONDAY
        | Weekdays::TUESDAY
        | Weekdays::WEDNESDAY
        | Weekdays::THURSDAY
        | Weekdays::FRIDAY;

    let rules: Vec<Rule<()>> = vec![
        Rule {
            start_date: NaiveDate::from_str("2000-01-01").unwrap(),
            end_date: NaiveDate::from_str("3000-01-01").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("00:00:00").unwrap(),
            weekdays: None,
            state: false,
            payload: None,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-01-01").unwrap(),
            end_date: NaiveDate::from_str("2025-01-01").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays_1),
            state: true,
            payload: None,
        },
    ];

    let start = NaiveDateTime::from_str("2024-12-24T08:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-12-24T16:00:00").unwrap();

    let frames = get_frames(&rules, start, end);
    assert_eq!(frames.len(), 1);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end,
            state: true,
            payload: None,
        }
    );
}

#[test]
fn test_is_within_weekday() {
    use chrono::NaiveDateTime;

    let weekdays = Weekdays::MONDAY
        | Weekdays::TUESDAY
        | Weekdays::WEDNESDAY
        | Weekdays::THURSDAY
        | Weekdays::FRIDAY;

    let time = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
    assert_eq!(super::is_within_weekdays(time.date(), weekdays), true);

    let time = NaiveDateTime::from_str("2024-04-30T12:00:00").unwrap();
    assert_eq!(super::is_within_weekdays(time.date(), weekdays), true);

    let time = NaiveDateTime::from_str("2024-05-01T12:00:00").unwrap();
    assert_eq!(super::is_within_weekdays(time.date(), weekdays), true);

    let time = NaiveDateTime::from_str("2024-05-02T12:00:00").unwrap();
    assert_eq!(super::is_within_weekdays(time.date(), weekdays), true);

    let time = NaiveDateTime::from_str("2024-05-03T12:00:00").unwrap();
    assert_eq!(super::is_within_weekdays(time.date(), weekdays), true);

    let time = NaiveDateTime::from_str("2024-05-04T12:00:00").unwrap();
    assert_eq!(super::is_within_weekdays(time.date(), weekdays), false);

    let time = NaiveDateTime::from_str("2024-05-05T12:00:00").unwrap();
    assert_eq!(super::is_within_weekdays(time.date(), weekdays), false);

    let time = NaiveDateTime::from_str("2024-05-06T12:00:00").unwrap();
    assert_eq!(super::is_within_weekdays(time.date(), weekdays), true);
}
