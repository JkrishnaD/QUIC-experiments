use crate::experiments::{connection, streams};

mod experiments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let exp = std::env::args().nth(1).expect("pass `connection`");

    match exp.as_str() {
        "connection" => connection::run().await.unwrap(),
        "stream" => streams::run().await.unwrap(),
        _ => panic!("Provide Recommended role"),
    };

    Ok(())
}
