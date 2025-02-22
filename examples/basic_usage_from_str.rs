use availability::availability::Availability;
use availability::rulebuilder::RuleBuilder;
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
        .start_time_str("2024-01-01 09:00:00") // 2024-01-01 09:00:00
        .end_time_str("2024-01-31 17:00:00") // 2024-01-31 17:00:00
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
        .start_time_str("2024-01-01 09:00:00") // 2024-01-01 09:00:00
        .end_time_str("2024-01-07 20:00:00") // 2024-01-07 20:00:00
        .weekdays(&["mon", "tue", "wed", "thu"])
        .payload(StoreHours {
            staff_count: 5,
            manager_on_duty: "Sale Team".to_string(),
        })
        .build()
        .unwrap();

    // Store closes for inventory checks (Priority 3 - highest priority)
    let inventory_day = RuleBuilder::new()
        .start_time_str("2024-01-15 00:00:00") // 2024-01-15 00:00:00
        .end_time_str("2024-01-16 00:00:00") // 2024-01-16 00:00:00
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
    store_availability.to_frames_in_range_str("2024-01-01 00:00:00", "2024-01-24 00:00:00");

    // Display the results
    println!("Store Schedule Overview:");
    println!("=======================");
    println!("{}", store_availability);
}
