pub const MONDAY: u8 = 1;
pub const TUESDAY: u8 = 2;
pub const WEDNESDAY: u8 = 4;
pub const THURSDAY: u8 = 8;
pub const FRIDAY: u8 = 16;
pub const SATURDAY: u8 = 32;
pub const SUNDAY: u8 = 64;
pub const ALL_WEEKDAYS: u8 = MONDAY | TUESDAY | WEDNESDAY | THURSDAY | FRIDAY | SATURDAY | SUNDAY;

pub fn get_days_from_mask(mask: u8) -> Vec<&'static str> {
    let mut days = Vec::new();
    if mask & MONDAY != 0 {
        days.push("monday");
    }
    if mask & TUESDAY != 0 {
        days.push("tuesday");
    }
    if mask & WEDNESDAY != 0 {
        days.push("wednesday");
    }
    if mask & THURSDAY != 0 {
        days.push("thursday");
    }
    if mask & FRIDAY != 0 {
        days.push("friday");
    }
    if mask & SATURDAY != 0 {
        days.push("saturday");
    }
    if mask & SUNDAY != 0 {
        days.push("sunday");
    }
    days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weekday_constants() {
        // Verify each constant is a power of 2
        assert_eq!(MONDAY, 0b00000001);
        assert_eq!(TUESDAY, 0b00000010);
        assert_eq!(WEDNESDAY, 0b00000100);
        assert_eq!(THURSDAY, 0b00001000);
        assert_eq!(FRIDAY, 0b00010000);
        assert_eq!(SATURDAY, 0b00100000);
        assert_eq!(SUNDAY, 0b01000000);

        // Verify no overlapping bits
        assert_eq!(MONDAY & TUESDAY, 0);
        assert_eq!(WEDNESDAY & THURSDAY, 0);
        assert_eq!(FRIDAY & SATURDAY, 0);
        assert_eq!(SUNDAY & MONDAY, 0);
    }

    #[test]
    fn test_get_days_from_mask_single_days() {
        assert_eq!(get_days_from_mask(MONDAY), vec!["monday"]);
        assert_eq!(get_days_from_mask(TUESDAY), vec!["tuesday"]);
        assert_eq!(get_days_from_mask(WEDNESDAY), vec!["wednesday"]);
        assert_eq!(get_days_from_mask(THURSDAY), vec!["thursday"]);
        assert_eq!(get_days_from_mask(FRIDAY), vec!["friday"]);
        assert_eq!(get_days_from_mask(SATURDAY), vec!["saturday"]);
        assert_eq!(get_days_from_mask(SUNDAY), vec!["sunday"]);
    }

    #[test]
    fn test_get_days_from_mask_multiple_days() {
        // Test weekdays (Monday through Friday)
        let weekdays = MONDAY | TUESDAY | WEDNESDAY | THURSDAY | FRIDAY;
        assert_eq!(
            get_days_from_mask(weekdays),
            vec!["monday", "tuesday", "wednesday", "thursday", "friday"]
        );

        // Test weekend
        let weekend = SATURDAY | SUNDAY;
        assert_eq!(get_days_from_mask(weekend), vec!["saturday", "sunday"]);

        // Test arbitrary combination
        let mon_wed_fri = MONDAY | WEDNESDAY | FRIDAY;
        assert_eq!(
            get_days_from_mask(mon_wed_fri),
            vec!["monday", "wednesday", "friday"]
        );
    }

    #[test]
    fn test_get_days_from_mask_edge_cases() {
        // Test empty mask
        assert_eq!(get_days_from_mask(0), Vec::<&str>::new());

        // Test all days
        let all_days = MONDAY | TUESDAY | WEDNESDAY | THURSDAY | FRIDAY | SATURDAY | SUNDAY;
        assert_eq!(
            get_days_from_mask(all_days),
            vec![
                "monday",
                "tuesday",
                "wednesday",
                "thursday",
                "friday",
                "saturday",
                "sunday"
            ]
        );

        // Test invalid bits (should ignore them)
        let mask_with_invalid_bits = MONDAY | WEDNESDAY | 0b10000000;
        assert_eq!(
            get_days_from_mask(mask_with_invalid_bits),
            vec!["monday", "wednesday"]
        );
    }

    #[test]
    fn test_get_days_from_mask_order() {
        // Test that days are always returned in the same order regardless of how the mask is constructed
        let forward = MONDAY | TUESDAY | WEDNESDAY;
        let reverse = WEDNESDAY | TUESDAY | MONDAY;

        assert_eq!(get_days_from_mask(forward), get_days_from_mask(reverse));

        assert_eq!(
            get_days_from_mask(forward),
            vec!["monday", "tuesday", "wednesday"]
        );
    }
}
