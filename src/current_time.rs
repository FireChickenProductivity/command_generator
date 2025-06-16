use std::time::{SystemTime, UNIX_EPOCH};

fn compute_time_in_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn compute_timestamp() -> String {
    let seconds = compute_time_in_seconds();
    format!("{}", seconds)
}
