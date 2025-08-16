//! A simple example using the crate's authorized websocket capabilities. Creates an HTTP and websocket
//! connection and subscribes to OwnTransactions.
//!
//! The handler function waits for a transaction to happen, then finds the address involved who
//! wasn't our user. It feeds this address to the client who fetches and logs the wallet.
//!
//! This is an example trying to demonstrate how you can compose the websocket event stream with
//! other components of your app. You could call a discord webhook on each transaction, or execute
//! a set of your own business logic on every transaction received from the stream.

use kromer_api::{
    Error,
    http::Client,
    model::{
        Address, PrivateKey, Wallet,
        ws::{SubscriptionType, WebSocketEvent},
    },
    ws::{Auth, WsClient, WsConfig},
};
use tokio::sync::mpsc::Receiver;
use tracing::{info, instrument, warn};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let pk = PrivateKey::from("PRIVATE KEY"); // CHANGEME
    let addr = Address::from(&pk);

    // let http = Client::new("http://localhost:8080")?;
    let http = Client::new("https://kromer.reconnected.cc")?;

    let cfg = WsConfig::new()
        .subscribe(SubscriptionType::OwnTransactions)
        .with_auth(pk);

    let (client, event_stream) = http.connnect_ws_config(cfg).await?;

    event_loop(addr, client, event_stream).await?;

    Ok(())
}

#[instrument(skip_all)]
async fn event_loop(
    addr: Address,
    client: WsClient<Auth>,
    mut rx: Receiver<WebSocketEvent>,
) -> Result<(), Error> {
    info!("waiting for transaction...");

    while let Some(event) = rx.recv().await {
        match event {
            WebSocketEvent::Transaction { transaction } => {
                let wallet: Wallet;
                if transaction.to == addr {
                    wallet = client.get_wallet(&transaction.to).await?;
                } else if let Some(from) = transaction.from
                    && from == addr
                {
                    wallet = client.get_wallet(&from).await?;
                } else {
                    continue;
                }
                info!("Other user's wallet: {wallet:#?}");
            }
            _ => warn!("received wrong event type!"),
        }
    }
    Ok(())
}
