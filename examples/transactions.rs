use kromer_api::{
    Error, KromerClient,
    endpoints::{Endpoint, krist::MakeTransactionEp},
    model::krist::{Address, PrivateKey},
};
use rust_decimal::Decimal;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let span = tracing::span!(Level::INFO, "main");
    let _guard = span.enter();

    let client = KromerClient::new("https://kromer.reconnected.cc")?;

    let pk = PrivateKey::new("PRIVATE KEY"); // Change this before running

    let ep = MakeTransactionEp::new_unchecked(Address::ServerWelf, Decimal::new(1, 2), pk)
        .set_meta("message=kromer-api example;".to_string());

    let trans = ep.query(&client).await?;

    println!("{trans:?}");

    Ok(())
}
