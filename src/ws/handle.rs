use crate::{model::ws::WebSocketEvent, ws::MalformedResponseSnafu};

use super::messages::{WebSocketMessage, WebSocketMessageInner};
use futures_util::{StreamExt, stream::SplitStream};
use scc::HashMap;
use snafu::ResultExt;
use std::{fmt::Debug, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{mpsc::Sender, oneshot},
};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use tracing::{debug, trace};
use tracing::{error, instrument, warn};

#[instrument(name = "handle_ws_incoming", skip_all)]
pub async fn handle_incoming(
    mut rx: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin + Debug>>,
    pending: Arc<HashMap<usize, oneshot::Sender<WebSocketMessageInner>>>,
    event_tx: Sender<WebSocketEvent>,
) {
    while let Some(res) = rx.next().await {
        // trace!("ws message: {res:?}");
        let msg = {
            match res {
                Ok(Message::Text(b)) => {
                    // trace!("ws text message: {}", b.as_str());

                    let res = serde_json::from_str::<WebSocketMessage>(b.as_str())
                        .context(MalformedResponseSnafu);

                    if let Ok(m) = res {
                        m
                    } else {
                        #[allow(clippy::unwrap_used)]
                        let err = res.unwrap_err();
                        warn!("Couldn't deserialize WS frame: {err:#?}");

                        continue;
                    }
                }
                Ok(Message::Ping(_)) => {
                    trace!("Received ping");
                    continue;
                }
                Ok(Message::Close(_)) => {
                    debug!("closing ws");
                    break;
                }

                Ok(_) => {
                    warn!("Encountered unexpected ws frame");
                    continue;
                }
                Err(e) => {
                    error!("Encountered ws error: {e}");
                    break;
                }
            }
        };

        match (msg.id, msg.msg) {
            (_, WebSocketMessageInner::Event { event }) => {
                let _ = event_tx.send(event).await;
            }
            (Some(n), inner) => {
                if let Some((_, os)) = pending.remove_async(&n).await
                    && os.send(inner).is_err()
                {
                    warn!("failed to pass message");
                }
            }
            // We ignore these two eitheir because they need to be handled, but not beyond
            // deserialization. KeepAlive is always bundled with a ping which we handle above.
            // Hello is only received on startup and we don't do anything with it.
            (None, WebSocketMessageInner::Hello | WebSocketMessageInner::KeepAlive) => (),
            (None, inner) => warn!("Received untagged response: {inner:#?}"),
        }
    }
}
