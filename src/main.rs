mod experiments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let exp = std::env::args().nth(1).expect("pass `connection`");

    match exp.as_str() {
        "connection" => experiments::run().await.unwrap(),
        _ => panic!("Provide Recommended role"),
    };

    Ok(())
}
