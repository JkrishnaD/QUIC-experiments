use std::{sync::Arc, time::Duration};

use quinn::{Endpoint, VarInt};

use crate::experiments::{SkipServerVerification, generate_certificate};

pub async fn run() -> anyhow::Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    tokio::spawn(flow_server());
    flow_client().await.unwrap();
    Ok(())
}

async fn flow_server() -> anyhow::Result<()> {
    let certificate = generate_certificate().unwrap();

    let mut tls_identity = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificate.cert_chain, certificate.private_key)?;

    tls_identity.alpn_protocols = vec![b"demo".to_vec()];

    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(tls_identity)?,
    ));

    let mut transport_config = quinn::TransportConfig::default();

    transport_config.stream_receive_window(VarInt::from_u32(4 * 1024));
    transport_config.receive_window(VarInt::from_u32(8 * 1024));

    server_config.transport = Arc::new(transport_config);

    let addr = "127.0.0.1:4433".parse().unwrap();
    let endpoint = Endpoint::server(server_config, addr)?;

    let incoming = endpoint.accept().await.unwrap();

    let connection = incoming.await.unwrap();
    let conn_start = tokio::time::Instant::now();

    println!(
        "[server] connection established at [+{:>2}ms]",
        conn_start.elapsed().as_millis()
    );

    loop {
        let (_send, mut recv) = connection.accept_bi().await.unwrap();

        tokio::spawn(async move {
            while let Some(chunk) = recv.read_chunk(1024, true).await.unwrap() {
                println!(
                    "[server] [+{:>3}ms] stream recieved {:?} bytes",
                    conn_start.elapsed().as_millis(),
                    chunk.bytes.len()
                );
                println!("[server] Waiting...");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });
    }
}

async fn flow_client() -> anyhow::Result<()> {
    let mut tls_identity = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
        .with_no_client_auth();

    tls_identity.alpn_protocols = vec![b"demo".to_vec()];

    let client_config = quinn::ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(tls_identity)?,
    ));

    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    let connecting = endpoint.connect("127.0.0.1:4433".parse()?, "localhost")?;

    let connection = connecting.await.unwrap();
    let conn_start = tokio::time::Instant::now();
    println!("[client] connection established");

    let handle = tokio::spawn(async move {
        let (mut send, _recv) = connection.open_bi().await.unwrap();

        let stream_id = send.id().index();
        println!(
            "[client] [+{:>3}ms] opened stream {} ",
            conn_start.elapsed().as_millis(),
            stream_id
        );

        for i in 0..10_000 {
            let msg = vec![b'x'; 1024]; // 1 KB

            let t0 = conn_start.elapsed().as_millis();
            send.write_all(&msg).await.unwrap();
            let t1 = conn_start.elapsed().as_millis();

            println!("[client] write {} took {} ms", i, t1 - t0);
        }

        send.finish().unwrap();
    });

    handle.await.unwrap();

    Ok(())
}
