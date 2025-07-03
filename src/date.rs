use {
    crate::lang::Localizer,
    serde::{Deserialize, Serialize},
    std::fmt::Write,
    time::{Month, OffsetDateTime},
};

pub fn now() -> Date {
    let date = OffsetDateTime::now_local()
        .inspect_err(|e| eprintln!("{e}"))
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .date();

    Date {
        day: date.day(),
        month: date.month(),
        year: date.year(),
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Date {
    pub day: u8,
    #[serde(with = "conv")]
    pub month: Month,
    pub year: i32,
}

impl Date {
    pub fn render(self, l: Localizer<'_>) -> impl maud::Render {
        struct Render<'loc>(Date, Localizer<'loc>);

        impl maud::Render for Render<'_> {
            fn render_to(&self, buffer: &mut String) {
                let Self(Date { day, month, year }, l) = self;
                let month_name = l.localize(month).unwrap_or_else(|| {
                    eprintln!("unknown month {month} for lang {}!", l.lang());
                    "nul"
                });

                _ = write!(buffer, "{day} {month_name} {year}");
            }
        }

        Render(self, l)
    }
}

mod conv {
    use {
        super::*,
        serde::{Deserializer, Serializer, de::Error},
    };

    #[allow(clippy::trivially_copy_pass_by_ref, reason = "serde derive API")]
    pub fn serialize<S>(&month: &Month, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(u8::from(month))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Month, D::Error>
    where
        D: Deserializer<'de>,
    {
        let u = u8::deserialize(deserializer)?;
        Month::try_from(u)
            .map_err(|e| D::Error::custom(format!("failed to deserialize month: {e}")))
    }
}
