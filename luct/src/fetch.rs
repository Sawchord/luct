use eyre::{Context, Report};
use luct_core::{Certificate, CertificateChain};
use rustls::pki_types::ServerName;
use rustls_platform_verifier::BuilderVerifierExt as _;
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
};
use url::Url;

// NOTE: This code is largely copied from https://github.com/robjtede/inspect-cert-chain/blob/main/src/fetch.rs

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
    let chain = tls
        .conn
        .peer_certificates()
        .map(|c| {
            CertificateChain::from(
                c.iter()
                    .filter_map(|c| Certificate::from_der(c).ok())
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap();

    Ok(chain)
}
