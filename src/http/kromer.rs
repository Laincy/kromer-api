use serde::Deserialize;

use crate::model::KromerError;

#[derive(Debug, Deserialize)]
pub struct KromerResponse<T> {
    pub data: Option<T>,
    pub error: Option<RawKromerError>,
}

impl<T> KromerResponse<T> {
    pub fn extract(self) -> Result<T, KromerError> {
        match (self.data, self.error) {
            (Some(res), None) => Ok(res),
            (None, Some(e)) => Err(e.into()),
            _ => Err(KromerError::InternalServerError {
                message: "Server returned an invalid response".to_string(),
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RawKromerError {
    pub code: String,
    pub message: String,
}

impl From<RawKromerError> for KromerError {
    fn from(value: RawKromerError) -> Self {
        let code: &str = &value.code;
        let message = value.message;

        match code {
            "resource_not_found_error" => Self::ResourceNotFoundError,
            "wallet_error" => Self::WalletError { message },
            "transaction_error" => Self::TransactionError { message },
            "player_error" => Self::PlayerError { message },
            _ => Self::InternalServerError { message },
        }
    }
}
