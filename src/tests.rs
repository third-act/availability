use std::str::FromStr;

use chrono::{NaiveDate, NaiveTime};

use crate::{
    frame::Frame,
    get_frames,
    rule::{Rule, Weekdays},
};

//////////////////////////////////////////
// Test with println! to see the output //
// cargo test -- --nocapture            //
//////////////////////////////////////////

// #[test]
// fn get_frames_two_rules_regular_opening_times() {
//     use chrono::NaiveDateTime;

//     let weekdays = Weekdays::MONDAY
//         | Weekdays::TUESDAY
//         | Weekdays::WEDNESDAY
//         | Weekdays::THURSDAY
//         | Weekdays::FRIDAY;

//     let rules = vec![
//         Rule {
//             start_date: NaiveDate::from_str("2000-01-01").unwrap(),
//             end_date: NaiveDate::from_str("3000-01-01").unwrap(),
//             start_time: NaiveTime::from_str("00:00:00").unwrap(),
//             end_time: NaiveTime::from_str("00:00:00").unwrap(),
//             weekdays: None,
//             state: false,
//         },
//         Rule {
//             start_date: NaiveDate::from_str("2024-01-01").unwrap(),
//             end_date: NaiveDate::from_str("2025-01-01").unwrap(),
//             start_time: NaiveTime::from_str("08:00:00").unwrap(),
//             end_time: NaiveTime::from_str("16:00:00").unwrap(),
//             weekdays: Some(weekdays),
//             state: true,
//         },
//         Rule {
//             start_date: NaiveDate::from_str("2024-06-03").unwrap(),
//             end_date: NaiveDate::from_str("2024-06-06").unwrap(),
//             start_time: NaiveTime::from_str("08:00:00").unwrap(),
//             end_time: NaiveTime::from_str("16:00:00").unwrap(),
//             weekdays: Some(weekdays),
//             state: false,
//         },
//     ];

//     let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
//     let end = NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap();

//     let frames = get_frames(rules, start, end);
//     assert_eq!(frames.len(), 1);
//     assert_eq!(
//         frames[0],
//         Frame {
//             start,
//             end,
//             state: true,
//         }
//     );
// }

// // ADD MORE TEST
// // 1. Test for the case when there is no rule
// // 2. Test for the case when there is only one rule
// // 3. Test for the case when there are multiple rules (>2)

// #[test]
// fn get_frames_no_rule() {
//     use chrono::NaiveDateTime;

//     let rules = vec![];

//     let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
//     let end = NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap();

//     let frames = get_frames(rules, start, end);
//     assert_eq!(frames.len(), 1);
//     assert_eq!(
//         frames[0],
//         Frame {
//             start,
//             end,
//             state: false,
//         }
//     );
// }

// #[test]
// fn get_frames_one_rule_with_baseline() {
//     use chrono::NaiveDateTime;

//     let weekdays = Weekdays::MONDAY
//         | Weekdays::TUESDAY
//         | Weekdays::WEDNESDAY
//         | Weekdays::THURSDAY
//         | Weekdays::FRIDAY;

//     let rules = vec![
//         Rule {
//             start_date: NaiveDate::from_str("2000-01-01").unwrap(),
//             end_date: NaiveDate::from_str("3000-01-01").unwrap(),
//             start_time: NaiveTime::from_str("00:00:00").unwrap(),
//             end_time: NaiveTime::from_str("00:00:00").unwrap(),
//             weekdays: None,
//             state: false,
//         },
//         Rule {
//             start_date: NaiveDate::from_str("2024-01-01").unwrap(),
//             end_date: NaiveDate::from_str("2025-01-01").unwrap(),
//             start_time: NaiveTime::from_str("08:00:00").unwrap(),
//             end_time: NaiveTime::from_str("16:00:00").unwrap(),
//             weekdays: Some(weekdays),
//             state: true,
//         },
//     ];

//     let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
//     let end = NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap();

//     let frames = get_frames(rules, start, end);
//     assert_eq!(frames.len(), 1);
//     assert_eq!(
//         frames[0],
//         Frame {
//             start,
//             end,
//             state: true,
//         }
//     );
// }

// #[test]
// fn get_frames_multiple_rules_scenario_1() {
//     use chrono::NaiveDateTime;

//     let weekdays_1 = Weekdays::MONDAY
//         | Weekdays::TUESDAY
//         | Weekdays::WEDNESDAY
//         | Weekdays::THURSDAY
//         | Weekdays::FRIDAY;

//     let weekdays_2 = Weekdays::SATURDAY | Weekdays::SUNDAY;

//     let weekdays_3 = Weekdays::MONDAY | Weekdays::TUESDAY;

//     let rules = vec![
//         Rule {
//             start_date: NaiveDate::from_str("2000-01-01").unwrap(),
//             end_date: NaiveDate::from_str("3000-01-01").unwrap(),
//             start_time: NaiveTime::from_str("00:00:00").unwrap(),
//             end_time: NaiveTime::from_str("00:00:00").unwrap(),
//             weekdays: None,
//             state: false,
//         },
//         Rule {
//             start_date: NaiveDate::from_str("2024-01-01").unwrap(),
//             end_date: NaiveDate::from_str("2025-01-01").unwrap(),
//             start_time: NaiveTime::from_str("08:00:00").unwrap(),
//             end_time: NaiveTime::from_str("16:00:00").unwrap(),
//             weekdays: Some(weekdays_1),
//             state: true,
//         },
//         Rule {
//             start_date: NaiveDate::from_str("2024-06-03").unwrap(),
//             end_date: NaiveDate::from_str("2024-06-06").unwrap(),
//             start_time: NaiveTime::from_str("08:00:00").unwrap(),
//             end_time: NaiveTime::from_str("16:00:00").unwrap(),
//             weekdays: Some(weekdays_3),
//             state: false,
//         },
//         Rule {
//             start_date: NaiveDate::from_str("2024-05-11").unwrap(),
//             end_date: NaiveDate::from_str("2025-05-12").unwrap(),
//             start_time: NaiveTime::from_str("08:00:00").unwrap(),
//             end_time: NaiveTime::from_str("03:00:00").unwrap(),
//             weekdays: Some(weekdays_2),
//             state: true,
//         },
//     ];

//     let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
//     let end = NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap();

//     let frames = get_frames(rules.clone(), start, end);
//     assert_eq!(frames.len(), 1);
//     assert_eq!(
//         frames[0],
//         Frame {
//             start,
//             end,
//             state: true,
//         }
//     );

//     let start = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
//     let end = NaiveDateTime::from_str("2024-04-29T18:00:00").unwrap();

//     let frames = get_frames(rules.clone(), start, end);
//     assert_eq!(frames.len(), 2);
//     assert_eq!(
//         frames[0],
//         Frame {
//             start,
//             end: NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap(),
//             state: true,
//         }
//     );
//     assert_eq!(
//         frames[1],
//         Frame {
//             start: NaiveDateTime::from_str("2024-04-29T16:00:00").unwrap(),
//             end,
//             state: false,
//         }
//     );

//     let start = NaiveDateTime::from_str("2024-06-04T12:00:00").unwrap();
//     let end = NaiveDateTime::from_str("2024-06-04T16:00:00").unwrap();

//     let frames = get_frames(rules.clone(), start, end);
//     assert_eq!(frames.len(), 1);
//     assert_eq!(
//         frames[0],
//         Frame {
//             start,
//             end,
//             state: false,
//         }
//     );

//     let start = NaiveDateTime::from_str("2024-05-07T03:00:00").unwrap();
//     let end = NaiveDateTime::from_str("2024-05-07T20:00:00").unwrap();

//     let frames = get_frames(rules, start, end);
//     // assert_eq!(frames.len(), 3);
//     assert_eq!(
//         frames[0],
//         Frame {
//             start,
//             end: NaiveDateTime::from_str("2024-05-07T08:00:00").unwrap(),
//             state: false,
//         }
//     );
//     assert_eq!(
//         frames[1],
//         Frame {
//             start: NaiveDateTime::from_str("2024-05-07T08:00:00").unwrap(),
//             end: NaiveDateTime::from_str("2024-05-07T16:00:00").unwrap(),
//             state: true,
//         }
//     );
//     assert_eq!(
//         frames[2],
//         Frame {
//             start: NaiveDateTime::from_str("2024-05-07T16:00:00").unwrap(),
//             end,
//             state: false,
//         }
//     );
// }

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

    let rules = vec![
        Rule {
            start_date: NaiveDate::from_str("2000-01-01").unwrap(),
            end_date: NaiveDate::from_str("3000-01-01").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("00:00:00").unwrap(),
            weekdays: None,
            state: false,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-01-01").unwrap(),
            end_date: NaiveDate::from_str("2025-01-01").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays_1),
            state: true,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-06-03").unwrap(),
            end_date: NaiveDate::from_str("2024-06-06").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("16:00:00").unwrap(),
            weekdays: Some(weekdays_3),
            state: false,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-05-11").unwrap(),
            end_date: NaiveDate::from_str("2025-05-11").unwrap(),
            start_time: NaiveTime::from_str("08:00:00").unwrap(),
            end_time: NaiveTime::from_str("23:59:59").unwrap(),
            weekdays: Some(weekdays_2),
            state: true,
        },
        Rule {
            start_date: NaiveDate::from_str("2024-05-12").unwrap(),
            end_date: NaiveDate::from_str("2025-05-12").unwrap(),
            start_time: NaiveTime::from_str("00:00:00").unwrap(),
            end_time: NaiveTime::from_str("03:00:00").unwrap(),
            weekdays: Some(weekdays_2),
            state: true,
        },
    ];

    let start = NaiveDateTime::from_str("2024-05-11T11:00:00").unwrap();
    let end = NaiveDateTime::from_str("2024-05-12T04:00:00").unwrap();

    let frames = get_frames(rules.clone(), start, end);
    assert_eq!(frames.len(), 2);
    assert_eq!(
        frames[0],
        Frame {
            start,
            end: NaiveDateTime::from_str("2024-05-12T03:00:00").unwrap(),
            state: true,
        }
    );
    assert_eq!(
        frames[1],
        Frame {
            start: NaiveDateTime::from_str("2024-05-12T03:00:00").unwrap(),
            end,
            state: false,
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

// #[test]
// fn test_is_within_time() {
//     use chrono::NaiveDateTime;

//     let rule = Rule {
//         start_date: NaiveDate::from_str("2000-01-01").unwrap(),
//         end_date: NaiveDate::from_str("3000-01-01").unwrap(),
//         start_time: NaiveTime::from_str("00:00:00").unwrap(),
//         end_time: NaiveTime::from_str("00:00:00").unwrap(),
//         weekdays: None,
//         state: false,
//     };

//     let time = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
//     assert_eq!(super::is_within_rule_time(time, &rule), true);

//     let time = NaiveDateTime::from_str("2024-04-29T07:00:00").unwrap();
//     assert_eq!(super::is_within_rule_time(time, &rule), true);

//     let weekdays_1 = Weekdays::MONDAY
//         | Weekdays::TUESDAY
//         | Weekdays::WEDNESDAY
//         | Weekdays::THURSDAY
//         | Weekdays::FRIDAY;

//     let rule = Rule {
//         start_date: NaiveDate::from_str("2000-01-01").unwrap(),
//         end_date: NaiveDate::from_str("3000-01-01").unwrap(),
//         start_time: NaiveTime::from_str("08:00:00").unwrap(),
//         end_time: NaiveTime::from_str("16:00:00").unwrap(),
//         weekdays: Some(weekdays_1),
//         state: true,
//     };

//     let time = NaiveDateTime::from_str("2024-04-29T15:00:00").unwrap();
//     assert_eq!(super::is_within_rule_time(time, &rule), true);

//     let time = NaiveDateTime::from_str("2024-04-29T12:00:00").unwrap();
//     assert_eq!(super::is_within_rule_time(time, &rule), true);

//     let time = NaiveDateTime::from_str("2024-04-29T18:00:00").unwrap();
//     assert_eq!(super::is_within_rule_time(time, &rule), false);

//     let time = NaiveDateTime::from_str("2024-04-29T20:00:00").unwrap();
//     assert_eq!(super::is_within_rule_time(time, &rule), false);

//     // let rule = Rule {
//     //     start: NaiveDateTime::from_str("2024-05-11T08:00:00").unwrap(),
//     //     end: NaiveDateTime::from_str("2024-05-12T03:00:00").unwrap(),
//     //     weekdays: Some(weekdays_1),
//     //     state: true,
//     // };

//     // let time = NaiveDateTime::from_str("2024-05-11T20:00:00").unwrap();
//     // assert_eq!(super::is_within_rule_time(time, &rule), true);

//     // let time = NaiveDateTime::from_str("2024-05-11T23:59:59").unwrap();
//     // assert_eq!(super::is_within_rule_time(time, &rule), true);

//     // let time = NaiveDateTime::from_str("2024-05-12T02:00:00").unwrap();
//     // assert_eq!(super::is_within_rule_time(time, &rule), true);

//     // let time = NaiveDateTime::from_str("2024-05-12T05:00:00").unwrap();
//     // assert_eq!(super::is_within_rule_time(time, &rule), false);
// }
