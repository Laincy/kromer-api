use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};

/// Errors emmitted when parsing a [`WalletPrivateKey`]
#[derive(Debug, thiserror::Error)]
pub enum WalletPkParseError {
    /// When the input is not 32 bytes
    #[error("input must be 32 bytes long, got {0}")]
    InvalidLen(usize),
    /// Occurs when a non alphanumeric character is passed to the parse function
    #[error("input myst be alphanumeric, '-', or '_'. Got {0}")]
    InvalidByte(u8),
}

/// A [`Wallet`](super::Wallet) private key
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WalletPrivateKey([u8; 32]);

impl WalletPrivateKey {
    /// Parses a slice of bytes into a [`WalletPrivateKey`]
    ///
    /// # Errors
    /// Errors if the input is too long or non alphanumeric
    pub const fn parse(val: &[u8]) -> Result<Self, WalletPkParseError> {
        let len = val.len();

        if len != 32 {
            return Err(WalletPkParseError::InvalidLen(len));
        }

        let mut bytes = [0u8; 32];

        let mut i = 0;

        while i < 32 {
            let b = val[i];
            match b {
                b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' => bytes[i] = b,
                _ => return Err(WalletPkParseError::InvalidByte(b)),
            }
            i += 1;
        }

        Ok(Self(bytes))
    }

    /// Returns a reference to the underlying bytes
    #[must_use]
    pub const fn inner(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for WalletPrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(self.inner()).expect("violated invariance"))
    }
}

impl Serialize for WalletPrivateKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for WalletPrivateKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct WalletPkVisitor;

        impl Visitor<'_> for WalletPkVisitor {
            type Value = WalletPrivateKey;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("wallet private key")
            }

            fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
                WalletPrivateKey::parse(v.as_bytes()).map_err(DeError::custom)
            }
        }

        deserializer.deserialize_any(WalletPkVisitor)
    }
}

impl TryFrom<&str> for WalletPrivateKey {
    type Error = WalletPkParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value.as_bytes())
    }
}
