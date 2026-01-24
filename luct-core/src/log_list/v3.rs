use crate::{CtLog, CtLogConfig, Version, utils::base64::Base64};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogList {
    version: String,
    log_list_timestamp: DateTime<Utc>,
    operators: Vec<Operators>,
}

impl LogList {
    pub fn currently_active_logs(&self) -> Vec<CtLog> {
        self.active_logs(Utc::now())
    }

    pub fn active_logs(&self, time: DateTime<Utc>) -> Vec<CtLog> {
        self.logs(
            // Check that the interval of included logs is not in the past.
            // If it is, this log can not contain certificates, that are still valid
            // and therefore we don't need to include it.
            |interval| {
                interval
                    .as_ref()
                    .is_some_and(|interval| interval.end_exclusive > time)
            },
            // Only logs in qualified, usable and readonly states should be considered active
            // See https://googlechrome.github.io/CertificateTransparency/log_states.html
            |state| {
                state.as_ref().is_some_and(|state| {
                    matches!(
                        state,
                        State::Qualified { .. } | State::Usable { .. } | State::Readonly { .. }
                    )
                })
            },
            // Logs that have no type, or that are marked Prod are considered active
            |log_type| {
                log_type
                    .as_ref()
                    .is_none_or(|log_type| matches!(log_type, LogType::Prod))
            },
        )
    }

    pub fn all_logs(&self) -> Vec<CtLog> {
        self.logs(|_| true, |_| true, |_| true)
    }

    fn logs<TF, SF, TYF>(&self, time_filter: TF, state_filter: SF, type_filter: TYF) -> Vec<CtLog>
    where
        TF: Fn(&Option<Interval>) -> bool,
        SF: Fn(&Option<State>) -> bool,
        TYF: Fn(&Option<LogType>) -> bool,
    {
        self.operators
            .iter()
            .flat_map(|op| op.logs.iter().chain(op.tiled_logs.iter()))
            .filter(|&log| time_filter(&log.temporal_interval))
            .filter(|&log| state_filter(&log.state))
            .filter(|&log| type_filter(&log.log_type))
            .filter_map(|log| {
                let config = CtLogConfig {
                    description: log.description.clone(),
                    version: Version::V1,
                    url: match &log.url {
                        LogUrl::Log { url } => url.clone(),

                        LogUrl::TiledLog { submission_url, .. } => submission_url.clone(),
                    },
                    tile_url: match &log.url {
                        LogUrl::Log { .. } => None,
                        LogUrl::TiledLog { monitoring_url, .. } => Some(monitoring_url.clone()),
                    },
                    key: log.key.clone(),
                    mmd: log.mmd,
                };
                let log = CtLog::new(config);

                if log.log_id() == &log.log_id {
                    Some(log)
                } else {
                    None
                }
            })
            .collect()
    }
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
    state: Option<State>,
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
#[serde(rename_all = "snake_case")]
enum State {
    Pending {
        timestamp: DateTime<Utc>,
    },
    Qualified {
        timestamp: DateTime<Utc>,
    },
    Usable {
        timestamp: DateTime<Utc>,
    },
    Readonly {
        timestamp: DateTime<Utc>,
        final_tree_head: FinalTreeHead,
    },
    Retired {
        timestamp: DateTime<Utc>,
    },
    Rejected {
        timestamp: DateTime<Utc>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FinalTreeHead {
    sha256_root_hash: Base64<Vec<u8>>,
    tree_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    const ALL_LOG_LIST: &str = include_str!("../../../testdata/all_logs_list.json");

    #[test]
    fn parse_log_list() {
        let time = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2025, 12, 14)
                    .unwrap()
                    .and_hms_milli_opt(1, 0, 0, 0)
                    .unwrap(),
            )
            .unwrap();

        let log_list: LogList = serde_json::from_str(ALL_LOG_LIST).unwrap();
        let all_logs = log_list.all_logs();

        assert_eq!(all_logs.len(), 187);
        // NOTE: This is the length including tiled logs
        //assert_eq!(all_logs.len(), 247);

        let active_logs = log_list.active_logs(time);

        assert_eq!(active_logs.len(), 45);
        // NOTE: This is the length including tiled logs
        //assert_eq!(active_logs.len(), 71);
    }
}
