//! Handle connections to the Kromer2 websocket API in a flexible manner
//!
//! The websocket client can be used to both listen to and query the Kromer2 server. This makes it
//! a powerful and fast way to communicate with the server. When creating a [`WsClient`], you
//! will receive an additional [`Receiver`], which will be fed [`WebSocketEvents`](`WebSocketEvent`).
//! While it is not as convenient as callbacks, it offers a more flexible solution.

use crate::model::{
    Address, PrivateKey, Wallet,
    ws::{SubscriptionType, WebSocketEvent},
};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use messages::{
    MessageResponseInner, WebSocketMessageInner, WebSocketRequest, WebSocketRequestInner,
};
use scc::HashMap;
use serde::Serialize;
use snafu::ResultExt;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    net::TcpStream,
    sync::{Mutex, mpsc::Receiver, oneshot},
    time::timeout,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use tracing::{error as terror, info, instrument, trace};

pub use error::*;

mod error;
mod handle;
mod messages;

/// A client for the Kromer2 websocket API
#[allow(dead_code)]
pub struct WsClient {
    pending_reqs: Arc<HashMap<usize, oneshot::Sender<WebSocketMessageInner>>>,
    /// The current message counter
    n: AtomicUsize,
    tx: Arc<Mutex<SplitSink<KromerStream, Message>>>,
}

type KromerStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

impl WsClient {
    pub(crate) fn new(stream: KromerStream) -> (Self, Receiver<WebSocketEvent>) {
        let (tx, rx) = stream.split();

        let res = Self {
            tx: Arc::new(Mutex::new(tx)),
            n: AtomicUsize::default(),
            pending_reqs: Arc::default(),
        };

        let (send, recv) = tokio::sync::mpsc::channel(20);

        tokio::spawn(handle::handle_incoming(rx, res.pending_reqs.clone(), send));

        trace!("Initialized WS Client");
        (res, recv)
    }

    fn next_id(&self) -> usize {
        self.n.fetch_add(1, Ordering::Relaxed)
    }

    async fn make_request(
        &self,
        req: WebSocketRequestInner,
    ) -> Result<WebSocketMessageInner, WebSocketError> {
        let id = self.next_id();

        let (tx, rx) = oneshot::channel::<WebSocketMessageInner>();

        // Collisions should never happen here so we just ignore it
        let _ = self.pending_reqs.insert_async(id, tx).await;

        let msg = WebSocketRequest { id, inner: req }.into_message();

        trace!("registered request {id}");
        let send_res = self.tx.lock().await.send(msg).await.context(WsNetSnafu);

        if let Err(e) = send_res {
            terror!("Couldn't receive request {id}");

            // Remove from queue to prevent leak
            self.pending_reqs.remove_async(&id).await;

            return Err(e);
        }

        // NOTE Timeout after 1s, maybe change or make a param when constructing WS connection
        timeout(Duration::from_secs(1), rx).await.map_or_else(
            |_| Err(WebSocketError::TimeOut),
            |v| v.map_err(|_| WebSocketError::RecvError),
        )
    }

    /// Subscribes the socket to a new [`SubscriptionType`]
    ///
    /// # Errors
    /// Errors if there is an issue with the underlying socket
    #[instrument(skip(self))]
    pub async fn subscribe(
        &self,
        event: SubscriptionType,
    ) -> Result<Vec<SubscriptionType>, WebSocketError> {
        let req = WebSocketRequestInner::Subscribe { event };

        let msg = self.make_request(req).await?;

        match msg {
            WebSocketMessageInner::Response {
                responding_to: MessageResponseInner::Subscribe { subscription_level },
            } => Ok(subscription_level),
            _ => Err(WebSocketError::InvalidType),
        }
    }

    /// Unsubscribes the socket from a [`SubscriptionType`]
    ///
    /// # Errors
    /// Errors if there is an issue with the underlying socket
    #[instrument(skip(self))]
    pub async fn unsubscribe(
        &self,
        event: SubscriptionType,
    ) -> Result<Vec<SubscriptionType>, WebSocketError> {
        let req = WebSocketRequestInner::Unsubscribe { event };

        let msg = self.make_request(req).await?;

        match msg {
            WebSocketMessageInner::Response {
                responding_to: MessageResponseInner::Subscribe { subscription_level },
            } => Ok(subscription_level),
            _ => Err(WebSocketError::InvalidType),
        }
    }

    /// DON'T USE THIS IT WILL ALWAYS TIME OUT
    ///
    /// # Errors
    /// Always errors, Kromer2 (for a reason I can't fathom) never responds to this but also
    /// doesn't send an error message. Same applies when getting valid subscription levels.
    pub async fn currently_subscribed(&self) -> Result<Vec<SubscriptionType>, WebSocketError> {
        let req = WebSocketRequestInner::GetSubscriptionLevel;

        let msg = self.make_request(req).await?;

        info!("current sub res: {msg:#?}");

        match msg {
            WebSocketMessageInner::Response {
                responding_to: MessageResponseInner::GetSubscriptionLevel { subscription_level },
            } => Ok(subscription_level),
            _ => Err(WebSocketError::InvalidType),
        }
    }

    /// Fetches the [`Wallet`] specified by [`Address`]
    ///
    /// # Errors
    /// Will error if there is an issue with the underlying socket
    ///
    /// See [`WebSocketError`] for more info
    #[instrument(skip(self))]
    pub async fn get_address(&self, addr: &Address) -> Result<Wallet, WebSocketError> {
        let req = WebSocketRequestInner::Address { address: *addr };
        let msg = self.make_request(req).await?;

        match msg {
            WebSocketMessageInner::Response {
                responding_to: MessageResponseInner::Address { address },
            } => Ok(address),
            _ => Err(WebSocketError::InvalidType),
        }
    }
}

/// A configuration for building a [`WsClient`]
#[derive(Debug, Serialize, Default)]
pub struct WsConfig {
    pk: Option<PrivateKey>,
    subscriptions: Vec<SubscriptionType>,
}

impl WsConfig {
    /// Creates a new [`Self`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pk: None,
            subscriptions: Vec::new(),
        }
    }

    /// Sets the [`PrivateKey`] to be used in authorization
    #[must_use]
    pub fn pk(mut self, pk: PrivateKey) -> Self {
        self.pk = Some(pk);
        self
    }

    /// Adds a subscription to [`Self`]
    #[must_use]
    pub fn subscribe(mut self, sub: SubscriptionType) -> Self {
        if !self.subscriptions.contains(&sub) {
            self.subscriptions.push(sub);
        }
        self
    }
}
