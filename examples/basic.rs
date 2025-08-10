use kromer_api::{
    Error, KromerClient,
    endpoints::{Endpoint, krist::GetWalletEp},
    model::krist::Address,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let client = KromerClient::new("https://kromer.reconnected.cc")?;

    let (wallet, _) = GetWalletEp::new(Address::ServerWelf).query(&client).await?;

    println!("serverwelf balance: {:?}", wallet.balance);

    Ok(())
}
