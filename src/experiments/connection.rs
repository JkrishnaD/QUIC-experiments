use std::{sync::Arc, time::Duration};

use quinn::Endpoint;
use rustls::{
    SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerifier},
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
};
use tokio::time;

pub async fn run() -> anyhow::Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    tokio::spawn(run_server());
    tokio::time::sleep(time::Duration::from_secs(1)).await;
    run_client().await?;
    Ok(())
}

async fn run_server() -> anyhow::Result<()> {
    let certificate = generate_certificate()?;

    let mut tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificate.cert_chain, certificate.private_key)?;

    tls_config.alpn_protocols = vec![b"demo".to_vec()];
    let server_config = quinn::ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(tls_config)?,
    ));

    let addr = "127.0.0.1:4433".parse().unwrap();
    let endpoint = Endpoint::server(server_config, addr)?;

    let incoming = endpoint.accept().await.unwrap();

    let connection = incoming.await.unwrap();
    println!("[server] Connection established");

    connection.closed().await;
    println!("[server] connection is closed");

    Ok(())
}

struct TlsIdentity {
    cert_chain: Vec<CertificateDer<'static>>,
    private_key: PrivateKeyDer<'static>,
}

fn generate_certificate() -> anyhow::Result<TlsIdentity> {
    let cert_key = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let cert_chain = vec![cert_key.cert.der().clone()];
    let private_key_der = cert_key.signing_key.serialize_der();
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(private_key_der));

    Ok(TlsIdentity {
        cert_chain,
        private_key,
    })
}

async fn run_client() -> anyhow::Result<()> {
    let mut tls_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();

    tls_config.alpn_protocols = vec![b"demo".to_vec()];

    let client_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(tls_config)?,
    ));

    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;

    endpoint.set_default_client_config(client_config);

    let connection = endpoint
        .connect("127.0.0.1:4433".parse()?, "localhost")?
        .await?;

    println!("[client] Connecting to server...");

    tokio::time::sleep(Duration::from_secs(2)).await;

    connection.close(0u32.into(), b"done");
    println!("[client] closed connection");

    Ok(())
}

#[derive(Debug)]
struct SkipServerVerification;

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ED25519,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PSS_SHA256,
        ]
    }
}
