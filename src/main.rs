use crate::experiments::{connection, flow_control, streams};

mod experiments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let exp = std::env::args()
        .nth(1)
        .expect("pass `connection` or `stream` or `flow`");

    match exp.as_str() {
        "connection" => connection::run().await.unwrap(),
        "stream" => streams::run().await.unwrap(),
        "flow" => flow_control::run().await.unwrap(),
        _ => panic!("Provide Recommended role"),
    };

    Ok(())
}
