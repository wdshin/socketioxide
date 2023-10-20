use std::{
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

use base64::Engine;
use rand::Rng;

/// A 128 bit session id type representing a base64 16 char string
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sid([u8; 16]);

impl Sid {
    /// Generate a new random session id (base64 10 chars)
    pub fn new() -> Self {
        Self::default()
    }

    fn as_str(&self) -> &str {
        // SAFETY: SID is always a base64 chars string
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SidDecodeError {
    #[error("Invalid url base64 string")]
    InvalidBase64String,
    #[error("Invalid sid length")]
    InvalidLength,
}

impl FromStr for Sid {
    type Err = SidDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use SidDecodeError::*;

        let mut id = [0u8; 16];

        // Verify the length of the string
        if s.len() != 16 {
            return Err(InvalidLength);
        }

        // Verify that the string is a valid base64 url safe string without padding
        for (idx, byte) in s.as_bytes()[0..16].iter().enumerate() {
            if (b'A'..=b'z').contains(byte)
                || (b'0'..=b'9').contains(byte)
                || byte == &b'_'
                || byte == &b'-'
            {
                id[idx] = *byte;
            } else {
                return Err(InvalidBase64String);
            }
        }
        Ok(Sid(id))
    }
}

impl Default for Sid {
    fn default() -> Self {
        let mut random = [0u8; 12]; // 12 bytes = 16 chars base64
        let mut id = [0u8; 16];

        rand::thread_rng().fill(&mut random);

        base64::prelude::BASE64_URL_SAFE_NO_PAD
            .encode_slice(random, &mut id)
            .unwrap();

        Sid(id)
    }
}

impl Display for Sid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl serde::Serialize for Sid {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

struct SidVisitor;
impl<'de> serde::de::Visitor<'de> for SidVisitor {
    type Value = Sid;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a valid sid")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Sid::from_str(v).map_err(serde::de::Error::custom)
    }
}
impl<'de> serde::Deserialize<'de> for Sid {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(SidVisitor)
    }
}

impl Debug for Sid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::sid::Sid;

    #[test]
    fn test_sid_from_str() {
        let id = Sid::new();
        let id2 = Sid::from_str(&id.to_string()).unwrap();
        assert_eq!(id, id2);
        let id = Sid::from_str("AA9AAA0AAzAAAAHs").unwrap();
        assert_eq!(id.to_string(), "AA9AAA0AAzAAAAHs");
    }

    #[test]
    fn test_sid_from_str_invalid() {
        let id = Sid::from_str("*$^ùù!").unwrap_err();
        assert_eq!(id.to_string(), "Invalid sid length");
        let id = Sid::from_str("aoassaAZDoin#zd{").unwrap_err();
        assert_eq!(id.to_string(), "Invalid url base64 string");
        let id = Sid::from_str("aoassaAZDoinazd<").unwrap_err();
        assert_eq!(id.to_string(), "Invalid url base64 string");
    }
}
