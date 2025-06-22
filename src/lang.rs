use {
    serde::{Deserialize, Serialize, de::Error as _},
    std::{collections::HashMap, fmt},
    time::Month,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lang([u8; 2]);

impl Lang {
    /// Creates a new lang value from an ascii string.
    ///
    /// Returns `Some` if two ascii chars are lowercase alphabetic,
    /// otherwise returns `None`.
    pub fn from_ascii(s: [u8; 2]) -> Option<Self> {
        if s.into_iter()
            .all(|c| c.is_ascii_lowercase() && c.is_ascii_alphabetic())
        {
            Some(Self(s))
        } else {
            None
        }
    }

    /// Creates a new lang value from a string.
    fn from_str(s: &str) -> Result<Self, Error> {
        let s = s.as_bytes().try_into().map_err(|_| Error::InvalidLen)?;
        Self::from_ascii(s).ok_or(Error::NonAsciiLowercaseAlphabetic)
    }

    fn as_str(&self) -> &str {
        str::from_utf8(&self.0).expect("ascii chars")
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Lang {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Lang {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(D::Error::custom)
    }
}

enum Error {
    NonAsciiLowercaseAlphabetic,
    InvalidLen,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonAsciiLowercaseAlphabetic => {
                f.write_str("non ascii lowercase alphabetic string")
            }
            Self::InvalidLen => f.write_str("invalid length"),
        }
    }
}

#[derive(Deserialize)]
pub struct Local(HashMap<Lang, Payload>);

impl Local {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn month_short_name(&self, month: Month, lang: Lang) -> Option<&str> {
        let payload = self.0.get(&lang)?;
        Some(&payload.month[month as usize - 1])
    }
}

#[derive(Deserialize)]
struct Payload {
    month: [Box<str>; 12],
}
