use std::{sync::Arc, time::Duration};

use quinn::Endpoint;
use tokio::time::{Instant, sleep};

use crate::experiments::{SkipServerVerification, generate_certificate};

pub async fn run() -> anyhow::Result<()> {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    tokio::spawn(stream_server());
    sleep(Duration::from_millis(200)).await;
    stream_client().await?;
    Ok(())
}

async fn stream_server() -> anyhow::Result<()> {
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

    let conn_start = std::time::Instant::now();
    println!(
        "[server] connection established at [+{:>2}ms]",
        conn_start.elapsed().as_millis()
    );

    loop {
        let (_send, mut recv) = connection.accept_bi().await.unwrap();

        tokio::spawn(async move {
            let stream_id = recv.id().index();

            println!(
                "[server] [+{:>3}ms] stream {} accepted",
                conn_start.elapsed().as_millis(),
                stream_id
            );
            while let Some(chunk) = recv.read_chunk(1024, true).await.unwrap() {
                println!(
                    "[server] [+{:>3}ms] stream {} recieved {:?} bytes",
                    conn_start.elapsed().as_millis(),
                    stream_id,
                    chunk.bytes
                )
            }

            println!(
                "[server] [+{:>3}ms] stream {} finished",
                conn_start.elapsed().as_millis(),
                stream_id
            )
        });
    }
}

async fn stream_client() -> anyhow::Result<()> {
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

    let connecting = endpoint
        .connect("127.0.0.1:4433".parse()?, "localhost")
        .unwrap();

    let connection = connecting.await.unwrap();
    let conn_start = Instant::now();
    println!("[client] connected");

    let delays = [200, 200, 200];

    for delay in delays {
        let conn = connection.clone();

        tokio::spawn(async move {
            let (mut send, _recv) = conn.open_bi().await.unwrap();

            let stream_id = send.id().index();
            println!(
                "[client] [+{:>3}ms] opened stream {} ",
                conn_start.elapsed().as_millis(),
                stream_id
            );

            for i in 0..3 {
                let msg = format!("chunk {}", i);
                send.write_all(&msg.as_bytes()).await.unwrap();
                sleep(Duration::from_millis(delay)).await;
            }
            send.finish().unwrap();
            println!(
                "[client] [+{:>3}ms] stream {:?} finished",
                conn_start.elapsed().as_millis(),
                stream_id
            );
        });
    }
    sleep(Duration::from_secs(5)).await;
    Ok(())
}
