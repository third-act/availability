use availability::{availability::Availability, rulebuilder::RuleBuilder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct StoreHours {
    staff_count: u32,
    manager_on_duty: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Create an empty Availability struct to store your schedule.
    let mut store_availability: Availability<StoreHours> = Availability::new();

    // 2) Define rules using RuleBuilder (string-based times in "YYMMDDHHMMSS" format).
    //    - Priority 1 (lowest): Open Mon-Fri from 09:00 to 17:00 (Jan 1 - Jan 31)
    let weekday_rule = RuleBuilder::new()
        .start_time_str("240101090000") // 2024-01-01 09:00:00
        .end_time_str("240131170000") // 2024-01-31 17:00:00
        .monday()
        .tuesday()
        .wednesday()
        .thursday()
        .friday()
        .payload(StoreHours {
            staff_count: 3,
            manager_on_duty: "Regular Manager".to_string(),
        })
        .build()?; // Validate & finalize

    //    - Priority 2: Special sale hours override (Jan 1 - Jan 7), open until 20:00
    let sale_rule = RuleBuilder::new()
        .start_time_str("240101090000") // 2024-01-01 09:00:00
        .end_time_str("240107200000") // 2024-01-07 20:00:00
        .weekdays(&["mon", "tue", "wed", "thu", "fri"])
        .payload(StoreHours {
            staff_count: 5,
            manager_on_duty: "Sale Team".to_string(),
        })
        .build()?;

    //    - Priority 3: Complete closure for inventory (Jan 15 - Jan 16)
    let inventory_rule = RuleBuilder::new()
        .start_time_str("240105000000") // 2024-01-15 00:00:00
        .end_time_str("240106000000") // 2024-01-16 00:00:00
        .off(true) // Off => store closed
        .payload(StoreHours {
            staff_count: 2,
            manager_on_duty: "Inventory Team".to_string(),
        })
        .build()?;

    // 3) Add the rules by ascending priority.
    store_availability.add_rule(weekday_rule, 1)?;
    store_availability.add_rule(sale_rule, 2)?;
    store_availability.add_rule(inventory_rule, 3)?;

    // 4) Convert the layered rules into "frames" that cover only the specified date range.
    store_availability.to_frames_in_range_str("240101000000", "240124235959");

    // Optional) Print out the resulting frames:
    println!("Store Schedule Overview:");
    println!("=======================");
    println!("{}", store_availability);

    // Optional) Get frame from datetime:
    println!("Get frames from datetime:");
    println!("=======================");
    let frame = store_availability
        .get_frame_from_str("240101090000")
        .unwrap();
    println!("Frame at 2024-01-01 09:00:00 is: {}", frame.off);
    if let Some(payload) = &frame.payload {
        println!("Staff Count: {}", payload.staff_count);
        println!("Manager on Duty: {}", payload.manager_on_duty);
    }

    Ok(())
}
