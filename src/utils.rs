use std::time::{SystemTime, UNIX_EPOCH};

pub fn format_timestamp_since(timestamp: u64) -> String {
    let seconds_since_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - timestamp;
    let unit: &str;
    let time = if seconds_since_timestamp >= 2*604800/*60*60*24*7*/ {
        unit = "weeks";
        seconds_since_timestamp/604800
    } else if seconds_since_timestamp >= 2*86400/*60*60*24*/ {
        unit = "days";
        seconds_since_timestamp/86400
    } else {
        unit = "hours";
        seconds_since_timestamp/3600
    };
    format!("Last updated {time} {unit} ago")
}


#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::utils::*;

    #[test]
    fn weeks_formatting() {
        for i in 2..10 {
            let result = format_timestamp_since(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - i*604800);
            assert_eq!(result, format!("Last updated {i} weeks ago"));
        }
    }
    #[test]
    fn weeks_formatting_1() {
        let result = format_timestamp_since(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 604800);
        assert_eq!(result, "Last updated 7 days ago");
    }
    
    #[test]
    fn days_formatting() {
        for i in 2..10 {
            let result = format_timestamp_since(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - i*86400);
            assert_eq!(result, format!("Last updated {i} days ago"));
        }
    }
    
    #[test]
    fn days_formatting_1() {
        let result = format_timestamp_since(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 86400);
        assert_eq!(result, "Last updated 24 hours ago");
    }
    
    #[test]
    fn hours_formatting() {
        for i in 2..43 {
            let result = format_timestamp_since(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - i*3600);
            assert_eq!(result, format!("Last updated {i} hours ago"));
        }
    }
    
    #[test]
    fn hours_formatting_1() {
        let result = format_timestamp_since(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - 3600);
        assert_eq!(result, "Last updated 1 hours ago");
    }
}
