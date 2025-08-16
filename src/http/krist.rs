use crate::{
    http::Paginator,
    model::{
        Address, PrivateKey, Wallet,
        krist::{KristError, NameInfo, Transaction, UnexpectedResponseSnafu},
    },
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ensure};

#[derive(Debug, Deserialize)]
pub struct RawKristError {
    pub error: String,
    pub message: String,
}

impl RawKristError {
    pub fn parse(self) -> Result<(), KristError> {
        let find_between = |first: &str, last: &str| -> Result<&str, KristError> {
            let word_start =
                first.len() + self.message.find(first).context(UnexpectedResponseSnafu)?;
            let word_end = self.message.find(last).context(UnexpectedResponseSnafu)?;

            Ok(&self.message[word_start..word_end])
        };

        Err(match self.error.as_str() {
            "address_not_found" => {
                let addr = find_between("Address ", " not found")?.to_string();

                KristError::AddrNotFound { addr }
            }
            "auth_failed" => KristError::AuthFailed,
            "name_not_found" => {
                let name = find_between("Name ", " not found")?.to_string();

                KristError::NameNotFound { name }
            }
            "name_taken" => {
                let name = find_between("Name ", " is already taken")?.to_string();

                KristError::NameTaken { name }
            }
            "not_name_owner" => {
                ensure!(self.message.len() > 30, UnexpectedResponseSnafu);

                let name = self.message[31..].to_string();

                KristError::NotNameOwner { name }
            }
            "insufficient_balance" | "insufficient_funds        ..user1" => KristError::InsufficientBalance,
            "transaction_not_found" => KristError::TransactionNotFound,
            "transactions_disabled" => KristError::TransactionsDisabled,
            "same_wallet_transfer" => KristError::SameWalletTransfer,
            "transaction_conflict" => {
                ensure!(self.message.len() > 35, UnexpectedResponseSnafu);

                let param = self.message[36..].to_string();

                KristError::TransactionConflict { param }
            }
            _ => KristError::InternalServerError {
                message: self.message,
            },
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct GetAddrRes {
    pub address: GetAddrInner,
}

#[derive(Debug, Deserialize)]
pub struct GetAddrInner {
    #[serde(flatten)]
    pub wallet: Wallet,
    #[serde(default)]
    pub names: u32,
}

#[derive(Debug, Serialize)]
pub struct AuthRequest<'a> {
    #[serde(rename = "privatekey")]
    pub pk: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct AuthRes {
    pub address: Address,
}

#[derive(Debug, Deserialize)]
pub struct SupplyRes {
    pub money_supply: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct NameRes {
    pub name: NameInfo,
}

#[derive(Debug, Deserialize)]
pub struct CostRes {
    pub name_cost: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct AvailRes {
    pub available: bool,
}

#[derive(Debug, Serialize)]
pub struct RegisterBody<'a> {
    pub privatekey: &'a PrivateKey,
}

#[derive(Debug, Serialize)]
pub struct TransferBody<'a> {
    pub address: &'a Address,
    pub privatekey: &'a PrivateKey,
}

#[derive(Debug, Serialize)]
pub struct UpdateBody<'a> {
    pub privatekey: &'a PrivateKey,
    pub a: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct ListTransactionsQuery<'a> {
    #[serde(rename = "excludeMined")]
    pub exclude_mined: bool,
    #[serde(flatten)]
    pub page: Option<&'a Paginator>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionRes {
    pub transaction: Transaction,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct MakeTransactionBody<'a> {
    pub privatekey: &'a PrivateKey,
    pub to: &'a Address,
    pub metadata: Option<&'a str>,
    pub amount: Decimal,
}
