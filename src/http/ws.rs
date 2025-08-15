use crate::{
    Error,
    model::ws::WebSocketEvent,
    ws::{WebSocketError, WsClient},
};
use rustls::{ClientConfig, RootCertStore};
use serde::Deserialize;
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::{Connector, connect_async_tls_with_config};
use tracing::instrument;
use url::Url;

use super::{Client, ClientMarker};

impl<M: ClientMarker> Client<M> {
    /// Start websocket session
    /// # Errors
    #[instrument(skip_all)]
    pub async fn connect_ws(&self) -> Result<(WsClient, Receiver<WebSocketEvent>), Error> {
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

        Ok(WsClient::new(stream))
    }
}

#[derive(Debug, Deserialize)]
struct WsConnRes {
    url: Url,
}
