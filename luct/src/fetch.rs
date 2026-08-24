use eyre::{Context, Report};
use luct_core::{Certificate, CertificateChain};
use rustls::pki_types::ServerName;
use rustls_platform_verifier::BuilderVerifierExt as _;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, LazyLock},
};
use url::Url;

// TODO: Ability to overwrite probe user agent
// TODO: Ability to set webpki verifier with custom roots instead of platform verifier
// TODO: Factor out into own crate
// TODO: Add error type

// NOTE: This code is copied and adapted from https://github.com/robjtede/inspect-cert-chain/blob/main/src/fetch.rs

static ROOTS: LazyLock<BTreeMap<String, Certificate>> = LazyLock::new(|| {
    webpki_root_certs::TLS_SERVER_ROOT_CERTS
        .iter()
        .map(|cert| Certificate::from_der(cert.as_ref()).unwrap())
        .map(|cert| (cert.get_subject(), cert))
        .collect()
});

pub(crate) fn fetch_cert_chain(url: &str) -> eyre::Result<CertificateChain> {
    let url = Url::parse(url).with_context(|| format!("failed to parse url: \"{url}\""))?;

    let server_name = ServerName::try_from(url.domain().unwrap())
        .with_context(|| format!("failed to convert given host (\"{url}\") to server name"))?
        .to_owned();

    let config = rustls::ClientConfig::builder_with_provider(rustls_rustcrypto::provider().into())
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_platform_verifier()?
        .with_no_client_auth();

    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name)?;

    let sock_addr = url.socket_addrs(|| None)?;
    let mut sock = TcpStream::connect(sock_addr[0])
        .wrap_err_with(|| format!("failed to connect to url: {url}"))?;
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);

    let req = format!(
        r#"GET / HTTP/1.1
Host: {url}
Connection: close
User-Agent: luct-cli/{}
Accept-Encoding: identity

"#,
        env!("CARGO_PKG_VERSION"),
    )
    .replace('\n', "\r\n");

    tracing::debug!("writing to socket:\n{req}");

    tls.write_all(req.as_bytes())
        .wrap_err("failed to write to socket")?;
    tls.flush().wrap_err("failed to flush socket")?;

    let mut plaintext = Vec::new();
    match tls.read_to_end(&mut plaintext) {
        Ok(_) => {}
        Err(err) => {
            tracing::warn!("failed to read from {url}: {}", Report::new(err));
        }
    }

    // peer_certificates method will return certificates by now
    // because app data has already been written
    // Unlike the browser extension, this call does not include the root certificate
    // We need to compile webpki-root-certs into the tool and find and match the root certificate here
    let chain = tls
        .conn
        .peer_certificates()
        .map(|certs| {
            certs
                .iter()
                .filter_map(|cert| Certificate::from_der(cert).ok())
                .collect::<Vec<_>>()
        })
        .map(|mut certs| {
            if let Some(root_cert) = certs.last().and_then(|cert| ROOTS.get(&cert.get_issuer())) {
                certs.push(root_cert.clone());
            }

            certs
        })
        .map(CertificateChain::from)
        .unwrap();

    Ok(chain)
}
