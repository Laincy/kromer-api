use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};

/// Errors thrown when parsing a [`WalletAddr`]
#[derive(Debug, thiserror::Error)]
pub enum WalletAddrParseError {
    /// Must start with a K
    #[error("input must start with a k")]
    StartWithK(u8),
    /// Occurs when the passed value's length is longer than 10
    #[error("address must eitheir be name or have a length of 10. Got {0}")]
    InvalidLen(usize),
    /// Occurs when a non alphanumeric character is passed to the parse function
    #[error("input must be a valid base36 character. Got {0}")]
    InvalidByte(u8),
}

/// An address that points to a specific [`Wallet`](super::Wallet)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WalletAddr {
    /// A normal 10 character address starting with "k"
    User(WalletAddrInner),
    /// The special `serverwelf` address
    ServerWelf,
    /// The special `name` address
    Name,
}

impl WalletAddr {
    pub(crate) const fn parse(bytes: &[u8]) -> Result<Self, WalletAddrParseError> {
        match bytes {
            b"serverwelf" => Ok(Self::ServerWelf),
            b"name" => Ok(Self::Name),
            _ => {
                let inner = WalletAddrInner::parse(bytes);

                match inner {
                    Ok(addr) => Ok(Self::User(addr)),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

impl std::fmt::Display for WalletAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(inner) => f.write_str(
                std::str::from_utf8(&inner.decode()).expect("violated WalletAddr's invariance"),
            ),
            Self::ServerWelf => f.write_str("serverwelf"),
            Self::Name => f.write_str("name"),
        }
    }
}

impl Serialize for WalletAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for WalletAddr {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct WalletAddrVisitor;

        impl Visitor<'_> for WalletAddrVisitor {
            type Value = WalletAddr;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("wallet address")
            }

            fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
                WalletAddr::parse(v.as_bytes()).map_err(DeError::custom)
            }
        }

        deserializer.deserialize_any(WalletAddrVisitor)
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct WalletAddrInner([u8; 9]);

impl WalletAddrInner {
    pub(crate) const fn parse(bytes: &[u8]) -> Result<Self, WalletAddrParseError> {
        if bytes.len() != 10 {
            return Err(WalletAddrParseError::InvalidLen(bytes.len()));
        }

        if bytes[0] != b'k' {
            return Err(WalletAddrParseError::StartWithK(bytes[0]));
        }

        let mut res = [0u8; 9];
        let mut i = 1;

        while i < 10 {
            let b = bytes[i];

            res[i - 1] = match b {
                // Or here since this will save us from bounds checks
                b'0'..=b'9' | b'a'..=b'z' => b,
                _ => return Err(WalletAddrParseError::InvalidByte(b)),
            };

            i += 1;
        }

        Ok(Self(res))
    }

    pub(crate) const fn decode(&self) -> [u8; 10] {
        let mut res = [b'k'; 10];

        let (_, addr) = res.split_at_mut(1);

        addr.copy_from_slice(&self.0);

        res
    }
}

#[test]
fn encode_decode_user() {
    let addr_parsed = WalletAddr::parse(b"kabcdefghi").unwrap();

    let res = format!("{addr_parsed}");

    assert_eq!(res, *"kabcdefghi");
}

#[test]
fn encode_decode_serverwelf() {
    let sw_parsed = WalletAddr::parse(b"serverwelf").unwrap();

    assert_eq!(sw_parsed, WalletAddr::ServerWelf);

    let res = format!("{sw_parsed}");

    assert_eq!(res, *"serverwelf");
}

#[test]
fn encode_decode_name() {
    let name_parsed = WalletAddr::parse(b"name").unwrap();

    assert_eq!(name_parsed, WalletAddr::Name);

    let res = format!("{name_parsed}");

    assert_eq!(res, *"name");
}
