use kromer_api::{
    Error,
    http::Client,
    model::{Address, PrivateKey},
};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let client = Client::new("https://kromer.reconnected.cc")?;

    let pk = PrivateKey::from("PRIVATE KEY"); // CHANGEME

    let addr = Address::try_from("ksg0aierdg")?;

    let res = client
        .make_transaction(&addr, Decimal::new(1, 2), Some("async in traits </3"), &pk)
        .await?;

    println!("{res:#?}");

    Ok(())
}
