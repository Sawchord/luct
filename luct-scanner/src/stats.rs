use crate::{Scanner, ScannerImpl, utils::system_time_to_date_time};
use luct_core::store::SearchableStoreRead;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web_time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicStatistics {
    roots_cas: Vec<(String, u64)>,
    scts: Vec<(String, u64)>,
}

impl<S: ScannerImpl> Scanner<S> {
    pub async fn basic_statistics(&self) -> BasicStatistics {
        let now = system_time_to_date_time(SystemTime::now());

        let reports = self
            .report_store
            .filter(|_, value| value.not_after > now)
            .await;

        let mut roots_cas = BTreeMap::new();
        let mut scts = BTreeMap::new();

        for (_, report) in reports.into_iter() {
            roots_cas
                .entry(report.ca_issuer)
                .and_modify(|entry| *entry += 1)
                .or_insert(1);

            for sct in report.scts {
                if let Some(name) = sct.log_name {
                    scts.entry(name)
                        .and_modify(|entry| *entry += 1)
                        .or_insert(1);
                }
            }
        }

        BasicStatistics {
            roots_cas: roots_cas.into_iter().collect(),
            scts: scts.into_iter().collect(),
        }
    }
}
