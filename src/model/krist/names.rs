use crate::model::{Address, BadSuffixSnafu, InvalidCharSnafu, LengthBoundsSnafu, ParseError};
use chrono::{DateTime, Utc};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};
use snafu::ensure;

/// A name object fetched from the Kromer2 API.
///
/// Does not include some fields defined in the Krist docs as these are irrelevant
/// for Kromer
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NameInfo {
    /// The name, without the `.kro` suffix
    pub name: String,
    /// The address that currently owns this name
    pub owner: Address,
    /// The address that originally purchased this name
    pub original_owner: Option<Address>,
    /// The date and time this name was registered
    pub registered: DateTime<Utc>,
    /// The date and time this name was last updated - eitheir the data changed, or it was transferred to a
    /// new owner
    pub updated: Option<DateTime<Utc>>,
    /// The date and time this name was last transferred to a new owner.
    pub transferred: Option<DateTime<Utc>>,
}

// begrudgingly heap allocate here because it actually makes sense. Though we do store it in a box
// tto save an extra usize of space
/// A name, stored without the `.kro` extension
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(Box<[u8]>);

impl Name {
    /// Creates a new name object. Note that this will clone at least part of the slice you pass
    /// in if it is successful.
    ///
    /// # Errors
    /// Errors if the input string is not an ascii alphanumeric character, '-', or '_' and has no
    /// extension besides an optional `.kro`
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let kro_i = s.find(".kro");

        let n_str: &str;

        if let Some(i) = kro_i {
            ensure!(s[i..] == *".kro", BadSuffixSnafu);
            n_str = &s[..i];
        } else {
            n_str = s;
        }

        ensure!(
            (1..=64).contains(&n_str.len()),
            LengthBoundsSnafu { len: n_str.len() }
        );

        for c in n_str.chars() {
            ensure!(
                matches!(c, 'a'..='z' |'A'..='Z' | '0'..='9' | '-' | '_'),
                InvalidCharSnafu { c }
            );
        }

        let mut inner: Box<[u8]> = Box::from(n_str.as_bytes());

        inner.make_ascii_lowercase();

        Ok(Self(inner))
    }

    /// Returns the underlying byte array as a string slice
    #[must_use]
    pub fn inner(&self) -> &str {
        // Safety: We can call unsafe Rust here since the bytes
        // of our Name being valid ASCII is one of our invariants
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner())
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct NameVisitor;

        impl Visitor<'_> for NameVisitor {
            type Value = Name;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("kromer name")
            }

            fn visit_str<E: DeError>(self, v: &str) -> Result<Self::Value, E> {
                Name::parse(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_any(NameVisitor)
    }
}

impl TryFrom<&str> for Name {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

/// A paginated list of [`Names`](Name) fetched from the Kromer2 API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamePage {
    /// The number of names recieved in this page
    pub count: usize,
    /// The total count of names
    pub total: usize,
    /// The page of names
    pub names: Vec<NameInfo>,
}
