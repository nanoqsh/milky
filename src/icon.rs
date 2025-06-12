use {serde::Deserialize, std::fmt};

#[derive(Clone, Copy, Deserialize)]
#[serde(try_from = "String")]
pub enum Icon {
    Discord,
    Github,
    X,
}

impl Icon {
    fn svg(self) -> &'static str {
        match self {
            Self::Discord => include_str!("../icons/discord.svg"),
            Self::Github => include_str!("../icons/github.svg"),
            Self::X => include_str!("../icons/x.svg"),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Discord => "Discord",
            Self::Github => "GitHub",
            Self::X => "Twitter",
        }
    }
}

impl TryFrom<String> for Icon {
    type Error = UnknownIcon;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "ds" => Ok(Self::Discord),
            "gh" => Ok(Self::Github),
            "x" => Ok(Self::X),
            _ => Err(UnknownIcon),
        }
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
