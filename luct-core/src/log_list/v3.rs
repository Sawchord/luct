use crate::utils::base64::Base64;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogList {
    version: String,
    log_list_timestamp: DateTime<Utc>,
    operators: Vec<Operators>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Operators {
    name: String,
    email: Vec<String>,
    logs: Vec<Logs>,
    tiled_logs: Vec<Logs>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Logs {
    description: String,
    log_id: Base64<Vec<u8>>,
    key: Base64<Vec<u8>>,

    mmd: u64,
    dns: Option<String>,
    // TODO: State
    temporal_interval: Option<Interval>,
    log_type: Option<LogType>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    previous_owners: Vec<PreviousOwner>,
    #[serde(flatten)]
    url: LogUrl,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum LogUrl {
    Log {
        url: Url,
    },
    TiledLog {
        submission_url: Url,
        monitoring_url: Url,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Interval {
    start_inclusive: DateTime<Utc>,
    end_exclusive: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LogType {
    Prod,
    Test,
    MonitoringOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PreviousOwner {
    name: String,
    end_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_log_list() {
        const LOG_LIST: &str = include_str!("../../../testdata/all_logs_list.json");

        let _: LogList = serde_json::from_str(LOG_LIST).unwrap();
    }
}
