use chrono::{DateTime, Utc};
use web_time::{SystemTime, UNIX_EPOCH};

pub(crate) fn system_time_to_date_time(time: SystemTime) -> DateTime<Utc> {
    DateTime::from_timestamp_millis(time.duration_since(UNIX_EPOCH).unwrap().as_millis() as i64)
        .unwrap()
}
