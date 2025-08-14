use kromer_api::{
    Error,
    http::{Client, Paginator},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let client = Client::new("https://kromer.reconnected.cc")?;

    let res = client.get_wallet_name("laincy").await?;

    println!("{res:#?}");

    let pg = Paginator::new(0, 1);

    let wallet = client
        .recent_wallet_transactions(&res[0].address, false, Some(&pg))
        .await?;

    println!("{wallet:#?}");

    Ok(())
}
