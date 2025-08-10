use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};
use sha2::{Digest, Sha256, digest::FixedOutput};
use std::fmt::Display;

/// A wallet fetched from the Kromer2 API
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Wallet {
    /// The address associated with this wallet
    pub address: Address,
    /// The amount of Kromer in this wallet
    pub balance: Decimal,
    /// The amount of Kromer that has ever recieved
    #[serde(alias = "totalin")]
    pub total_in: Decimal,
    /// The amount of Kromer that this wallet has ever paid out
    #[serde(alias = "totalout")]
    pub total_out: Decimal,
    /// The date and time of this wallet's first transaction
    #[serde(alias = "firstseen")]
    pub first_seen: DateTime<Utc>,
    // Ignore the names field as we handle this as a tuple later
}

/// A page of wallets fetched from a paginated API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletPage {
    /// The number of wallets returned in this query
    pub count: usize,
    /// The total number of wallets fetchable from the endpoint this value came from
    pub total: usize,
    /// The wallets fetched from this query
    #[serde(alias = "addresses")]
    pub wallets: Vec<Wallet>,
}

/// Errors thrown when parsing a value into a [`Address`] or [`PrivateKey`]
#[derive(Debug, snafu::Snafu)]
pub enum WalletParseError {
    /// Thrown when input exceeds the desired length
    #[snafu(display("exp {exp} bytes, got found {got}"))]
    InvalidLen {
        /// The length expected
        exp: u8,
        /// The length recieved
        got: usize,
    },
    /// Thrown when the input is not the special name `serverwelf` and doesn't start with a 'k
    #[snafu(display("expected bytes starting with 107 ('k'), found {got}"))]
    InvalidPrefix {
        /// The byte found
        got: u8,
    },
    /// Thrown when the input contains bytes that are not in the ranges 1-9 or a-z
    #[snafu(display(
        "expected a byte in ranges 46..=57 or 97..=122, found {got} at index {index} "
    ))]
    InvalidByte {
        /// The byte found
        got: u8,
        /// The index of the input at which the wrong byte was found
        index: usize,
    },
}

/// An address pointing to a wallet on the Krist API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Address {
    /// A normal user wallet in the format `^k[a-z0-9]{9}`
    User(AddressInner),
    /// The special `serverwelf` wallet
    ServerWelf,
}

impl Address {
    /// Parses a slice of bytes into `[Address]`
    ///
    /// # Errors
    /// Errors if the input is not a valid Krist wallet address See [`AddressParseError`] for
    /// more info
    pub const fn parse(bytes: &[u8]) -> Result<Self, WalletParseError> {
        // Allowing this here because the suggested replacement does not work in const environment
        #[allow(clippy::single_match_else)]
        match bytes {
            b"serverwelf" => Ok(Self::ServerWelf),
            _ => {
                if bytes.len() != 10 {
                    return Err(WalletParseError::InvalidLen {
                        exp: 10,
                        got: bytes.len(),
                    });
                } else if bytes[0] != b'k' {
                    return Err(WalletParseError::InvalidPrefix { got: bytes[0] });
                }

                let mut res = [0u8; 9];

                let mut i = 1;

                while i < 10 {
                    let b = bytes[i];

                    res[i - 1] = match b {
                        b'0'..=b'9' | b'a'..=b'z' => b,
                        _ => {
                            return Err(WalletParseError::InvalidByte { got: b, index: i });
                        }
                    };

                    i += 1;
                }

                Ok(Self::User(AddressInner(res)))
            }
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(inner) => {
                use std::fmt::Write;
                f.write_char('k')?;

                // Safety: We can call unsafe Rust here since the bytes
                // of inner being valid ASCII is one of our invariants
                let s = unsafe { std::str::from_utf8_unchecked(&inner.0) };

                f.write_str(s)
            }
            Self::ServerWelf => f.write_str("serverwelf"),
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

impl TryFrom<&[u8]> for Address {
    type Error = WalletParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for Address {
    type Error = WalletParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value.as_bytes())
    }
}

impl TryFrom<String> for Address {
    type Error = WalletParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc(hidden)]
pub struct AddressInner([u8; 9]);

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

impl From<PrivateKey> for Address {
    fn from(pk: PrivateKey) -> Self {
        // Possibly rewrite this later to use closures that write to the same arrays every time?
        // One for the 32 byte normal hash and one for the 64 byte hex hash

        let mut protein = [0u8; 9];
        let mut used = [false; 9];

        let mut chain = [0u8; 9];

        let mut hash = double_sha256(pk.0.as_bytes());

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

        Self::User(AddressInner(chain))
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
