use eyre::{Context, Report};
use luct_core::{Certificate, CertificateChain};
use rustls::{
    SignatureScheme,
    pki_types::{CertificateDer, ServerName, UnixTime},
};
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

    let mut config =
        rustls::ClientConfig::builder_with_provider(rustls_rustcrypto::provider().into())
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_platform_verifier()?
            .with_no_client_auth();

    config
        .dangerous()
        .set_certificate_verifier(Arc::new(NoopServerCertVerifier));

    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name)?;

    let sock_addr = url.socket_addrs(|| None)?;
    let mut sock = TcpStream::connect(sock_addr[0])
        .wrap_err_with(|| format!("failed to connect to url: {url}"))?;
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);

    let req = format!(
        r#"GET / HTTP/1.1
Host: {url}
Connection: close
User-Agent: inspect-cert-chain/{}
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

#[derive(Debug)]
struct NoopServerCertVerifier;

impl rustls::client::danger::ServerCertVerifier for NoopServerCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}
