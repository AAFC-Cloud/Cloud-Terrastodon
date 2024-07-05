use iso8601_duration::Duration as IsoDuration;
use std::time::Duration;

pub fn to_iso8601(duration: Duration) -> IsoDuration {
    let total_seconds = duration.as_secs();
    let seconds = (total_seconds % 60) as f32;
    let total_minutes = total_seconds / 60;
    let minutes = (total_minutes % 60) as f32;
    let total_hours = total_minutes / 60;
    let hours = (total_hours % 24) as f32;
    let days = (total_hours / 24) as f32;

    IsoDuration::new(0.0, 0.0, days, hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_iso8601() {
        let duration = Duration::new(28800, 0); // 8 hours in seconds
        let iso_string = to_iso8601(duration);
        assert_eq!(iso_string, "PT8H".parse().unwrap());
        
        let duration = Duration::new(3661, 0); // 1 hour, 1 minute, and 1 second
        let iso_string = to_iso8601(duration);
        assert_eq!(iso_string, "PT1H1M1S".parse().unwrap());

        let duration = Duration::new(90061, 0); // 1 day, 1 hour, 1 minute, and 1 second
        let iso_string = to_iso8601(duration);
        assert_eq!(iso_string, "P1DT1H1M1S".parse().unwrap());
    }
}
