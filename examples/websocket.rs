//! A simple example using the crate's websocket capabilities. Creates an HTTP and websocket
//! connection and subscribes to Transcation events while unsubscribing from other default events.
//!
//! The handler function waits for a transaction to happen, then takes the receipient of the
//! transaction and queries their recent transactions using the HTTP api. The "total" field of the
//! transaction page is then logged as a tracing statement.
//!
//! This is an example trying to demonstrate how you can compose the websocket event stream with
//! other components of your app. You could call a discord webhook on each transaction, or execute
//! a set of your own business logic on every transaction received from the stream.

use kromer_api::{
    Error,
    http::{Client, ClientMarker, Paginator},
    model::ws::{SubscriptionType, WebSocketEvent},
};
use tokio::sync::mpsc::Receiver;
use tracing::{debug, info, instrument, warn};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let http = Client::new("https://kromer.reconnected.cc")?;

    let (client, rx) = http.connect_ws().await?;

    // Subscribe to all transaction event
    for sub in client
        .subscribe(SubscriptionType::Transactions)
        .await?
        .into_iter()
        .filter(|x| *x != SubscriptionType::Transactions)
    {
        let _ = client.unsubscribe(sub).await?;
    }

    handle_events(http, rx).await
}

/// Waits for the first transaction to come through on the socket, and then finds out how many
/// total transactions the given user has made
#[instrument(skip_all)]
async fn handle_events(
    http: Client<impl ClientMarker>,
    mut rx: Receiver<WebSocketEvent>,
) -> Result<(), Error> {
    let page = Paginator::new(0, 1);

    info!("waiting for transaction...");
    while let Some(event) = rx.recv().await {
        debug!("recieved event: {event:#?}");

        match event {
            WebSocketEvent::Transaction { transaction } => {
                let addr = transaction.to;
                let recent = http
                    .recent_wallet_transactions(&addr, false, Some(&page))
                    .await?;

                info!(
                    "wallet {addr} has participated in {} transactions",
                    recent.total
                );

                return Ok(());
            }
            _ => warn!("Received event we're not subscribed to"),
        }
    }

    Ok(())
}
