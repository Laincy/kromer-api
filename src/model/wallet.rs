use super::ParseError;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};
use sha2::{Digest, Sha256, digest::FixedOutput};
use std::fmt::Write;
use std::fmt::{Debug, Display};

/// An address for a [`Wallet`] on the Kromer API
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum Address {
    /// A normal user wallet in the format `^k[a-z0-9]{9}`
    Normal(AddressInner),
    /// The special `serverwelf` wallet
    Serverwelf,
}

impl Address {
    /// Parses a slice of bytes into `[Address]`
    ///
    /// # Errors
    /// Errors if the input is not a valid Kromer address. See [`ParseError`]
    /// for more info
    pub const fn parse(bytes: &[u8]) -> Result<Self, ParseError> {
        // Allowing this here because the suggested replacement does not work in const environment
        #[allow(clippy::single_match_else)]
        match bytes {
            b"serverwelf" => Ok(Self::Serverwelf),
            _ => {
                if bytes.len() != 10 {
                    return Err(ParseError::UnexpectedLength {
                        exp: 10,
                        got: bytes.len(),
                    });
                } else if bytes[0] != b'k' {
                    return Err(ParseError::InvalidPrefix { got: bytes[0] });
                }

                let mut res = [0u8; 9];

                let mut i = 1;
                while i < 10 {
                    let b = bytes[i];

                    res[i - 1] = match b {
                        b'0'..=b'9' | b'a'..=b'z' => b,
                        _ => {
                            return Err(ParseError::InvalidByte { got: b, index: i });
                        }
                    };

                    i += 1;
                }

                Ok(Self::Normal(AddressInner(res)))
            }
        }
    }

    fn parse_pk(pk: &str) -> Self {
        let mut protein = [0u8; 9];
        let mut used = [false; 9];

        let mut chain = [0u8; 9];

        let mut hash = double_sha256(pk.as_bytes());

        for amino in &mut protein {
            *amino = from_radix(&hash[0..=1]);
            hash = double_sha256(&hash);
        }

        let mut i = 0;

        while i < 9 {
            let start = i * 2;
            let end = start + 2;
            let index = (from_radix(&hash[start..end]) % 9) as usize;

            if used[index] {
                hash = sha256(&hash);
            } else {
                chain[i] = hex_to_base36(protein[index]);
                used[index] = true;
                i += 1;
            }
        }

        Self::Normal(AddressInner(chain))
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal(inner) => {
                f.write_char('k')?;

                // Safety: We can call unsafe Rust here since the bytes
                // of inner being valid ASCII is one of our invariants
                let s = unsafe { std::str::from_utf8_unchecked(&inner.0) };

                f.write_str(s)
            }
            Self::Serverwelf => write!(f, "serverwelf"),
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal(inner) => {
                f.write_char('k')?;

                // Safety: We can call unsafe Rust here since the bytes
                // of inner being valid ASCII is one of our invariants
                let s = unsafe { std::str::from_utf8_unchecked(&inner.0) };

                f.write_str(s)
            }
            Self::Serverwelf => f.write_str("serverwelf"),
        }
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct AddressVisitor;

        impl Visitor<'_> for AddressVisitor {
            type Value = Address;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("wallet address")
            }

            fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
                Address::parse(v.as_bytes()).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_any(AddressVisitor)
    }
}

impl From<PrivateKey> for Address {
    fn from(value: PrivateKey) -> Self {
        Self::parse_pk(&value.0)
    }
}

impl From<&PrivateKey> for Address {
    fn from(value: &PrivateKey) -> Self {
        Self::parse_pk(&value.0)
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = ParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for Address {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value.as_bytes())
    }
}

impl TryFrom<String> for Address {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_bytes())
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd)]
pub struct AddressInner([u8; 9]);

/// A wallet fetched from the Kromer2 API. Does not include the ID field as
/// there is little use for it and omitting it will allow the same type to be
/// used for both the Kromer and Krist endpoints
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Wallet {
    /// The [`Address`] associated with the wallet
    pub address: Address,
    /// The amount of Kromer in this wallet
    pub balance: Decimal,
    /// When this wallet was created
    #[serde(alias = "firstseen")]
    pub created_at: DateTime<Utc>,
    /// Whether this wallet is stopped from making transactions. If the API does
    /// not include this field, will default to `false`
    #[serde(default)]
    pub locked: bool,
    /// The total amount of Kromer that has been sent to this wallet
    #[serde(alias = "totalin")]
    pub total_in: Decimal,
    /// The total amount of Kromer that has been sent from this wallet
    #[serde(alias = "totalout")]
    pub total_out: Decimal,
}

/// A private key for a specific [`Address`]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PrivateKey(Box<str>);

impl PrivateKey {
    /// Creates a new [`PrivateKey`]
    #[must_use]
    pub fn new(val: &str) -> Self {
        Self(Box::from(val))
    }

    /// Returns a reference to the underlying bytes
    #[must_use]
    pub const fn inner(&self) -> &str {
        &self.0
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for PrivateKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

fn sha256(bytes: &[u8]) -> [u8; 64] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    encode_hex(hasher.finalize_fixed().into())
}

fn double_sha256(data: &[u8]) -> [u8; 64] {
    sha256(&sha256(data))
}

fn encode_hex(bytes: [u8; 32]) -> [u8; 64] {
    const HEX_VALS: [u8; 16] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e',
        b'f',
    ];

    let mut res = [0u8; 64];

    for (i, b) in bytes.iter().enumerate() {
        res[2 * i] = HEX_VALS[(b >> 4) as usize];
        res[(2 * i) + 1] = HEX_VALS[(b & 0xf) as usize];
    }

    res
}

fn decode_hex(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => unreachable!(),
    }
}

fn from_radix(bytes: &[u8]) -> u8 {
    let mut res = decode_hex(bytes[1]);
    res |= decode_hex(bytes[0]) << 4;
    res
}

fn hex_to_base36(byte: u8) -> u8 {
    match byte / 7 {
        byte @ 0..=9 => byte + b'0',
        byte @ 10..=35 => byte + b'a' - 10,
        36 => b'e',
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::{Address, PrivateKey};

    #[test]
    fn parse_pk() {
        let correct = Address::parse(b"kdk1ku9oeq").unwrap();

        let pk = PrivateKey::new("y5HvW0g1wboIbLQaT6W3Wt8sT3f8tYO9");

        let maybe = Address::from(pk);

        assert_eq!(correct, maybe);
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct PkWrapper {
        pk: PrivateKey,
    }

    #[test]
    fn deser_pk() {
        serde_json::from_str::<PkWrapper>(
            r#"{
                "pk": "y5HvW0g1wboIbLQaT6W3Wt8sT3f8tYO9"
            }"#,
        )
        .unwrap();
    }
}
