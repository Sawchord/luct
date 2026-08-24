use chrono::{DateTime, FixedOffset, Utc};
use luct_core::CertificateChain;
use rustls::{
    client::{WebPkiServerVerifier, danger::ServerCertVerifier},
    pki_types::{CertificateDer, UnixTime},
};
use std::sync::Arc;
use web_time::Duration;

pub fn verify_cert_chain(chain: &CertificateChain, name: &str, time: &str) {
    // TODO: Ability to add more roots
    let client_verifier = WebPkiServerVerifier::builder(Arc::new(
        webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect(),
    ))
    //.allow_unauthenticated()
    .build()
    .unwrap();

    // FIXME: This should also work with web_time

    let time: DateTime<Utc> = DateTime::<FixedOffset>::parse_from_rfc2822(time)
        .unwrap()
        .into();
    let now = UnixTime::since_unix_epoch(Duration::from_secs(time.timestamp() as u64));

    let cert = CertificateDer::from(chain.cert().as_der());

    let mut intermediates = vec![];
    for idx in 1..chain.len() - 1 {
        intermediates.push(CertificateDer::from(chain[idx].as_der()));
    }

    client_verifier
        .verify_server_cert(
            &cert,
            &intermediates,
            &rustls::pki_types::ServerName::DnsName(name.try_into().unwrap()),
            &[],
            now,
        )
        .unwrap();
}
