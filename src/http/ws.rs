use crate::{
    Error,
    model::{PrivateKey, ws::WebSocketEvent},
    ws::{Guest, WebSocketError, WsClient, WsConfig, WsState},
};
use rustls::{ClientConfig, RootCertStore};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::{Connector, connect_async_tls_with_config};
use tracing::instrument;
use url::Url;

use super::{Client, ClientMarker};

impl<M: ClientMarker> Client<M> {
    /// Start websocket session, creating a [`WsClient`]. By default, this will be subscribed to
    /// nothing. Consider using the [`Self::connnect_ws_config`] method instead if you know what
    /// events you'd like to be subscribed to.
    ///
    /// # Errors
    /// Will error if the client cannot be created
    #[instrument(skip_all)]
    pub async fn connect_ws(&self) -> Result<(WsClient<Guest>, Receiver<WebSocketEvent>), Error> {
        let url = self
            .krist_post::<WsConnRes>("/api/krist/ws/start", ())
            .await?
            .url;

        let root_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };

        let connector = Connector::Rustls(
            ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()
                .into(),
        );

        let (stream, _) = connect_async_tls_with_config(url.as_str(), None, false, Some(connector))
            .await
            .map_err(|err| WebSocketError::WsNetError {
                source: Box::from(err),
            })?;

        Ok(WsClient::new(stream).await)
    }

    /// Start a websocket session, constructing it using [`WsConfig`].
    ///
    /// # Errors
    /// Errors if there is an issue constructing the socket
    pub async fn connnect_ws_config<S: WsState>(
        &self,
        cfg: WsConfig<S>,
    ) -> Result<(WsClient<S>, Receiver<WebSocketEvent>), Error> {
        let url = self
            .krist_post::<WsConnRes>("/api/krist/ws/start", WsConnBody { privatekey: cfg.pk })
            .await?
            .url;

        let root_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };

        let connector = Connector::Rustls(
            ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()
                .into(),
        );

        let (stream, _) = connect_async_tls_with_config(url.as_str(), None, false, Some(connector))
            .await
            .map_err(|err| WebSocketError::WsNetError {
                source: Box::from(err),
            })?;

        Ok(WsClient::<S>::new_from_config(stream, &cfg.subscriptions).await)
    }
}

#[derive(Debug, Deserialize)]
struct WsConnRes {
    url: Url,
}

#[derive(Debug, Serialize)]
struct WsConnBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    privatekey: Option<PrivateKey>,
}
