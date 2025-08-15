use snafu::Snafu;
use tokio_tungstenite::tungstenite::Error as TgError;

/// Errors thrown when working with the Kromer2 websocket API
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
#[allow(missing_docs)]
pub enum WebSocketError {
    /// Other end of the channel was dropped
    #[snafu(display("Couldn't receive result from channel"))]
    RecvError,
    /// Issues communicating with the websocket server
    #[snafu(display("Experienced issue when communicating with the WS server"))]
    WsNetError {
        #[snafu(source(from(TgError, Box::new)))]
        source: Box<TgError>,
    },
    /// Couldn't deserialize value into a model
    #[snafu(display("Failed to deserialize response into model"))]
    MalformedResponse { source: serde_json::Error },
    /// Type of message received did not align with the expected response type for the request
    #[snafu(display("Recieved incorrect response type for id"))]
    InvalidType,
    /// Request timed out
    #[snafu(display("Pending future timed out"))]
    TimeOut,
}
