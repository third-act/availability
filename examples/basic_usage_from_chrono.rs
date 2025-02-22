use availability::availability::Availability;
use availability::rulebuilder::RuleBuilder;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

fn main() {
    // Define a payload type for store hours
    #[derive(Serialize, Deserialize, Clone)]
    struct StoreHours {
        staff_count: u32,
        manager_on_duty: String,
    }

    let mut store_availability: Availability<StoreHours> = Availability::new();

    // Regular weekday hours (Priority 1)
    let weekday_hours = RuleBuilder::new()
        .start_datetime(create_datetime(2024, 1, 1, 9, 0, 0))
        .end_datetime(create_datetime(2024, 1, 31, 17, 0, 0))
        .monday()
        .tuesday()
        .wednesday()
        .thursday()
        .friday()
        .payload(StoreHours {
            staff_count: 3,
            manager_on_duty: "Regular Staff".to_string(),
        })
        .build() // <-- final validation
        .unwrap(); // unwrap final result

    // Extended hours for New Year's sale (Priority 2 - overrides regular hours)
    let new_year_sale = RuleBuilder::new()
        .start_datetime(create_datetime(2024, 1, 1, 9, 0, 0))
        .end_datetime(create_datetime(2024, 1, 7, 20, 0, 0))
        .weekdays(&["mon", "tue", "wed", "thu"])
        .payload(StoreHours {
            staff_count: 5,
            manager_on_duty: "Sale Team".to_string(),
        })
        .build()
        .unwrap();

    // Store closes for inventory checks (Priority 3 - highest priority)
    let inventory_day = RuleBuilder::new()
        .start_datetime(create_datetime(2024, 1, 15, 0, 0, 0))
        .end_datetime(create_datetime(2024, 1, 16, 0, 0, 0))
        // No weekdays specified makes it an absolute rule
        .off(true)
        .payload(StoreHours {
            staff_count: 2,
            manager_on_duty: "Inventory Team".to_string(),
        })
        .build()
        .unwrap();

    // Add rules in order of priority (lowest to highest)
    store_availability.add_rule(weekday_hours, 1).unwrap();
    store_availability.add_rule(new_year_sale, 2).unwrap();
    store_availability.add_rule(inventory_day, 3).unwrap();

    // Convert rules to frames between 2024-01-01 and 2024-01-24
    let start = create_datetime(2024, 1, 1, 0, 0, 0);
    let end = create_datetime(2024, 1, 24, 0, 0, 0);
    store_availability.to_frames_in_range(start, end);

    // Display the results
    println!("Store Schedule Overview:");
    println!("=======================");
    println!("{}", store_availability);
}

fn create_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(hour, minute, second)
        .unwrap()
}
