use {
    serde::{Deserialize, de},
    std::fmt,
};

#[derive(Clone, Copy)]
pub enum Icon {
    Discord,
    Github,
    X,
    Tooltip,
}

impl Icon {
    fn from_str(s: &str) -> Result<Self, UnknownIcon> {
        match s {
            "ds" => Ok(Self::Discord),
            "gh" => Ok(Self::Github),
            "x" => Ok(Self::X),
            "tt" => Ok(Self::Tooltip),
            _ => Err(UnknownIcon),
        }
    }

    fn svg(self) -> &'static str {
        match self {
            Self::Discord => include_str!("../icons/discord.svg"),
            Self::Github => include_str!("../icons/github.svg"),
            Self::X => include_str!("../icons/x.svg"),
            Self::Tooltip => include_str!("../icons/tooltip.svg"),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Discord => "Discord",
            Self::Github => "GitHub",
            Self::X => "Twitter",
            Self::Tooltip => "Tooltip",
        }
    }
}

impl<'de> Deserialize<'de> for Icon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visit;

        impl de::Visitor<'_> for Visit {
            type Value = Icon;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an icon")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Icon::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(Visit)
    }
}

pub struct UnknownIcon;

impl fmt::Display for UnknownIcon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown icon")
    }
}

impl maud::Render for Icon {
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(self.svg());
    }
}
