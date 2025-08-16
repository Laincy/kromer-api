//! Handle connections to the Kromer2 websocket API in a flexible manner
//!
//! The websocket client can be used to both listen to and query the Kromer2 server. This makes it
//! a powerful and fast way to communicate with the server. When creating a [`WsClient`], you
//! will receive an additional [`Receiver`], which will be fed [`WebSocketEvents`](`WebSocketEvent`).
//! While it is not as convenient as callbacks, it offers a more flexible solution.
//! ```rust
//! # use kromer_api::{Error, http::Client, model::Address};
//! # async fn run() -> Result<(), Error> {
//! let http = Client::new("https//kromer.reconnected.cc")?;
//! let (client, _event_rx) = http.connect_ws().await?;
//!
//! let wallet = client.get_wallet(&Address::Serverwelf).await?;
//!
//! println!("{wallet:#?}");
//! # Ok(())
//! # }
//! ```
//!
//! # Authorization
//! The websockets *do not* support logging in and out mid session. I personally, don't see a use
//! case for this that I find compelling enough to support. By not doing this, a [`WsClient`]
//! is tagged with a [`WsState`], indicating whether it's been authorized or not. This tag is set
//! on creation. By default, it will be a [`Guest`] which means it cannot make requests that rely
//! on being authorized.
//!
//! By creating a socket with a [`WsConfig`] that's had the [`WsConfig::with_auth`] method called
//! on it, you will receive an [`Auth`] client with additional capabilities.

use crate::{
    Error,
    http::RawKristError,
    model::{
        Address, PrivateKey, Wallet,
        krist::{SameWalletTransferSnafu, Transaction},
        ws::{SubscriptionType, WebSocketEvent},
    },
};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use messages::{
    MessageResponseInner, WebSocketMessageInner, WebSocketRequest, WebSocketRequestInner,
};
use rust_decimal::Decimal;
use scc::HashMap;
use serde::Serialize;
use snafu::{ResultExt, ensure};
use std::{
    marker::PhantomData,
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
use tracing::{debug, error as terror, instrument, trace};

pub use error::*;

mod error;
mod handle;
mod messages;

// Only reason we don't automatically impl WsState for all who have WsStateSealed is so that
// implementors will appear in docs
/// Designates the current state of the socket.
/// See its  for more info
#[allow(private_bounds)]
pub trait WsState: WsStateSealed + Send + Sync {}

pub(super) trait WsStateSealed {}

/// Indicates an unauthorized [`WsClient`] connection
pub struct Guest;
impl WsStateSealed for Guest {}
impl WsState for Guest {}

/// Indicates an authorized [`WsClient`] connection
pub struct Auth;
impl WsStateSealed for Auth {}
impl WsState for Auth {}

/// A client for the Kromer2 websocket API
#[allow(dead_code)]
pub struct WsClient<M: WsState> {
    pending_reqs: Arc<HashMap<usize, oneshot::Sender<WebSocketMessageInner>>>,
    /// The current message counter
    n: AtomicUsize,
    tx: Arc<Mutex<SplitSink<KromerStream, Message>>>,

    _marker: PhantomData<M>,
}

type KromerStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

impl<M: WsState> WsClient<M> {
    /// Closes the underlying socket
    ///
    /// # Errors
    /// Errors if the socket decides not to close for god knows what reason
    pub async fn close(self) -> Result<(), WebSocketError> {
        debug!("closing socket");

        let mut tx = self.tx.lock().await;

        tx.feed(Message::Close(None)).await.context(WsNetSnafu)?;

        let _ = tx.close().await;

        drop(tx);

        Ok(())
    }

    #[instrument(skip_all)]
    pub(crate) async fn new(stream: KromerStream) -> (Self, Receiver<WebSocketEvent>) {
        let (tx, rx) = stream.split();

        let res = Self {
            tx: Arc::new(Mutex::new(tx)),
            n: AtomicUsize::default(),
            pending_reqs: Arc::default(),
            _marker: PhantomData,
        };

        let (send, recv) = tokio::sync::mpsc::channel(20);

        tokio::spawn(handle::handle_incoming(rx, res.pending_reqs.clone(), send));

        let _ = tokio::join!(
            res.unsubscribe(SubscriptionType::Blocks),
            res.unsubscribe(SubscriptionType::OwnTransactions),
        );

        trace!("Initialized WS Client");
        (res, recv)
    }

    #[instrument(skip_all)]
    pub(crate) async fn new_from_config(
        stream: KromerStream,
        subs: &[SubscriptionType],
    ) -> (Self, Receiver<WebSocketEvent>) {
        let default_events = [SubscriptionType::Blocks, SubscriptionType::OwnTransactions];

        let (tx, rx) = stream.split();

        let res = Self {
            tx: Arc::new(Mutex::new(tx)),
            n: AtomicUsize::default(),
            pending_reqs: Arc::default(),
            _marker: PhantomData,
        };

        let (send, recv) = tokio::sync::mpsc::channel(20);

        tokio::spawn(handle::handle_incoming(rx, res.pending_reqs.clone(), send));

        for i in default_events.into_iter().filter(|v| !subs.contains(v)) {
            let _ = res.unsubscribe(i).await;
        }

        for i in subs.iter().filter(|v| !default_events.contains(v)) {
            let _ = res.subscribe(*i).await;
        }

        (res, recv)
    }

    fn next_id(&self) -> usize {
        self.n.fetch_add(1, Ordering::Relaxed)
    }

    async fn make_request(
        &self,
        req: WebSocketRequestInner<'_>,
    ) -> Result<MessageResponseInner, Error> {
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

            return Err(Error::WebsocketError { source: e });
        }

        // NOTE Timeout after 5s, maybe change or make a param when constructing WS connection
        let req_res = timeout(Duration::from_secs(10), rx).await.map_or_else(
            |_| Err(WebSocketError::TimeOut),
            |v| v.map_err(|_| WebSocketError::RecvError),
        )?;

        match req_res {
            WebSocketMessageInner::Error { error, message } => {
                RawKristError { error, message }.parse()?;
                unreachable!()
            }
            WebSocketMessageInner::Response { responding_to } => Ok(responding_to),
            _ => Err(WebSocketError::InvalidType.into()),
        }
    }

    /// Subscribes the socket to a new [`SubscriptionType`]
    ///
    /// # Errors
    /// Errors if there is an issue with the underlying socket
    #[instrument(skip(self))]
    pub async fn subscribe(&self, event: SubscriptionType) -> Result<Vec<SubscriptionType>, Error> {
        let req = WebSocketRequestInner::Subscribe { event };

        let msg = self.make_request(req).await?;

        match msg {
            MessageResponseInner::Subscribe { subscription_level } => Ok(subscription_level),
            _ => Err(WebSocketError::InvalidType.into()),
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
    ) -> Result<Vec<SubscriptionType>, Error> {
        let req = WebSocketRequestInner::Unsubscribe { event };

        let msg = self.make_request(req).await?;

        match msg {
            MessageResponseInner::Subscribe { subscription_level } => Ok(subscription_level),
            _ => Err(WebSocketError::InvalidType.into()),
        }
    }

    /// DON'T USE THIS IT WILL ALWAYS TIME OUT
    ///
    /// # Errors
    /// Always errors, Kromer2 (for a reason I can't fathom) never responds to this but also
    /// doesn't send an error message. Same applies when getting valid subscription levels.
    #[allow(dead_code)]
    #[instrument(skip(self))]
    pub async fn currently_subscribed(&self) -> Result<Vec<SubscriptionType>, Error> {
        let req = WebSocketRequestInner::GetSubscriptionLevel;

        let msg = self.make_request(req).await?;

        match msg {
            MessageResponseInner::GetSubscriptionLevel { subscription_level } => {
                Ok(subscription_level)
            }

            _ => Err(WebSocketError::InvalidType.into()),
        }
    }

    /// Fetches the [`Wallet`] specified by [`Address`]
    ///
    /// # Errors
    /// Will error if there is an issue with the underlying socket
    ///
    /// See [`WebSocketError`] for more info
    #[instrument(skip(self))]
    pub async fn get_wallet(&self, addr: &Address) -> Result<Wallet, Error> {
        let req = WebSocketRequestInner::Address { address: *addr };
        let msg = self.make_request(req).await?;

        match msg {
            MessageResponseInner::Address { address } => Ok(address),
            _ => Err(WebSocketError::InvalidType.into()),
        }
    }

    /// Makes a Kromer [`Transaction`]. Note that this does preform several
    /// expensive hashes to convert a [`PrivateKey`] into an [`Address`] to
    /// ensure they are not the same as `addr`
    ///
    /// # Arguments
    /// * `addr` - The [`Address`] the transaction is going to
    /// * `amount` - The amount of Kromer to sent
    /// * `meta` - The metadata to attach to this transaction
    /// * `pk` - The [`PrivateKey`] attached to the wallet sending the transaction
    ///
    /// # Errors
    /// Errors if both addresses are the same, or the wallet `pk` points to has
    /// insufficient funds.
    ///
    /// See [`WebSocketError`] for more info
    #[instrument(skip(self))]
    pub async fn make_transaction(
        &self,
        addr: &Address,
        amount: Decimal,
        meta: Option<&str>,
        pk: &PrivateKey,
    ) -> Result<Transaction, Error> {
        ensure!(Address::from(pk) != *addr, SameWalletTransferSnafu);

        let req = WebSocketRequestInner::MakeTransaction {
            privatekey: Some(pk),
            to: addr,
            metadata: meta,
            amount,
        };

        let msg = self.make_request(req).await?;

        match msg {
            MessageResponseInner::MakeTransaction { transaction } => Ok(transaction),
            _ => Err(WebSocketError::InvalidType.into()),
        }
    }
}

impl WsClient<Auth> {
    /// Makes a Kromer [`Transaction`], using the currently authorized user's private key to send
    /// the transaction.
    ///
    /// # Arguments
    /// * `addr` - The [`Address`] the transaction is going to
    /// * `amount` - The amount of Kromer to sent
    /// * `meta` - The metadata to attach to this transaction
    ///
    /// # Errors
    /// Errors if the wallets are the same or has insufficient funds or the transaction is being
    /// made to the same wallet that is authorized.
    ///
    /// See [`WebSocketError`] for more info
    pub async fn make_transaction_authed(
        &self,
        addr: &Address,
        amount: Decimal,
        meta: Option<&str>,
    ) -> Result<Transaction, Error> {
        let req = WebSocketRequestInner::MakeTransaction {
            privatekey: None,
            to: addr,
            metadata: meta,
            amount,
        };

        let msg = self.make_request(req).await?;

        match msg {
            MessageResponseInner::MakeTransaction { transaction } => Ok(transaction),
            _ => Err(WebSocketError::InvalidType.into()),
        }
    }

    /// Fetches information about the currently authorized user
    ///
    /// # Errors
    ///  Errors if there is an issue with the underlying socket, or if Kromer2 decides on its own
    ///  to break my type based invariant.
    pub async fn me(&self) -> Result<Wallet, Error> {
        match self.make_request(WebSocketRequestInner::Me).await? {
            MessageResponseInner::Me { address } => Ok(address),
            _ => Err(WebSocketError::InvalidType.into()),
        }
    }
}

/// A configuration for building a [`WsClient`]
#[derive(Debug, Serialize, Default)]
pub struct WsConfig<M: WsState> {
    pub(crate) pk: Option<PrivateKey>,
    pub(crate) subscriptions: Vec<SubscriptionType>,
    _marker: PhantomData<M>,
}

impl<M: WsState> WsConfig<M> {
    /// Adds a subscription to [`Self`]
    #[must_use]
    pub fn subscribe(mut self, sub: SubscriptionType) -> Self {
        if !self.subscriptions.contains(&sub) {
            self.subscriptions.push(sub);
        }
        self
    }
}

impl WsConfig<Guest> {
    /// Creates a new [`Self`]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pk: None,
            subscriptions: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Configures the websocket to be authorized on creation.
    /// This is the only way to do this, as we do not support logging in/out at run time.
    #[must_use]
    pub fn with_auth(self, pk: PrivateKey) -> WsConfig<Auth> {
        WsConfig::<Auth> {
            pk: Some(pk),
            subscriptions: self.subscriptions,
            _marker: PhantomData,
        }
    }
}
