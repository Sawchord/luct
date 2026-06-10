use crate::{Report, Scanner, ScannerImpl};
use chrono::{DateTime, TimeDelta, Utc};
use web_time::Duration;

impl<S: ScannerImpl> Scanner<S> {
    pub(crate) fn evaluate_policy(
        &self,
        mut report: Report,
        current_time: DateTime<Utc>,
    ) -> Report {
        // TODO: Check that expiration date matches logs expiration bracket?

        // Calculate the number of scts we expect
        let num_expected_scts = match report.not_after - report.not_before {
            time if time <= TimeDelta::days(180) => 2,
            _ => 3,
        };

        // Calculate the number of scts that the report contains from known logs
        // TODO: Make sure that the logs are from different operators
        // TODO: Use log ids here and check that logs are not retired. This has better UI, as it might
        // also recognize retired logs
        let num_scts_from_known_logs = report
            .scts
            .iter()
            // NOTE: Having a signature that passed validation means the log is known
            .filter(|sct| sct.signature_validation_time.is_some())
            .count();

        // Check that we have enough SCTs from known logs
        if num_scts_from_known_logs < num_expected_scts {
            return report.error_description(format!(
                "Insufficient number of SCTs from known logs. Expected {} but got {}",
                num_expected_scts, num_scts_from_known_logs
            ));
        }

        let mut fresh_inclusion_proofs = 0;
        let mut old_inclusion_proofs = 0;
        for sct in report.scts.iter_mut() {
            // Scts with error cannot be valid
            if sct.error_description.is_some() {
                continue;
            }

            // Check that the SCT has a a fresh STH
            let Some(latest_sth) = &sct.latest_sth else {
                sct.set_error_description("Could not find a fresh STH for this SCT".to_string());
                continue;
            };
            if latest_sth.verification_time
                < current_time - time_delta_from_duration(self.config.sth_freshness_threshold)
            {
                sct.set_error_description(
                    "This logs latest STH is too old and the log is considered stale".to_string(),
                );
                continue;
            }

            // Check whether the proofs are old or fresh
            let Some(inclusion_proof) = &sct.inclusion_proof else {
                // Could not find an inclusion proof for this SCT
                continue;
            };

            if inclusion_proof.verification_time
                < current_time - time_delta_from_duration(self.config.sth_freshness_threshold)
            {
                old_inclusion_proofs += 1;
            } else {
                fresh_inclusion_proofs += 1;
            }
        }

        if old_inclusion_proofs == 0 && fresh_inclusion_proofs < num_expected_scts {
            return report.error_description(
                "Insufficient number of inclusion proofs with fresh sths could be verified!"
                    .to_string(),
            );
        }

        report
    }
}

fn time_delta_from_duration(duration: Duration) -> TimeDelta {
    TimeDelta::new(duration.as_secs() as i64, duration.subsec_nanos())
        .expect("Failed to translate duration into timedelta")
}
