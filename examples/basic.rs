use kromer_api::{
    Error, KromerClient,
    endpoints::{Endpoint, krist::GetWalletEp},
    model::krist::Address,
};
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let span = tracing::span!(Level::INFO, "main");
    let _guard = span.enter();

    let client = KromerClient::new("https://kromer.reconnected.cc")?;
    let addr = Address::try_from("ksg0aierdg")?;
    let (wallet, _names) = GetWalletEp::new(addr).query(&client).await?;

    println!("{} balance: {:?}", addr, wallet.balance);

    Ok(())
}
