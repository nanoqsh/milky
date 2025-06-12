use {
    serde::Deserialize,
    std::fmt::Write,
    time::{Date, Month},
};

#[derive(Clone, Copy, Deserialize)]
pub enum Lang {
    #[serde(rename = "ru")]
    Ru,
    #[serde(rename = "en")]
    En,
}

pub fn render_date(date: Date, lang: Lang) -> impl maud::Render {
    struct Render(Date, Lang);

    impl maud::Render for Render {
        fn render_to(&self, buffer: &mut String) {
            let Self(date, lang) = self;
            _ = write!(
                buffer,
                "{} {} {}",
                date.day(),
                month_short_name(date.month(), *lang),
                date.year()
            );
        }
    }

    Render(date, lang)
}

fn month_short_name(month: Month, lang: Lang) -> &'static str {
    match lang {
        Lang::Ru => match month {
            Month::January => "янв",
            Month::February => "фев",
            Month::March => "мар",
            Month::April => "апр",
            Month::May => "май",
            Month::June => "июн",
            Month::July => "июл",
            Month::August => "авг",
            Month::September => "сен",
            Month::October => "окт",
            Month::November => "ноя",
            Month::December => "дек",
        },
        Lang::En => match month {
            Month::January => "jan",
            Month::February => "feb",
            Month::March => "mar",
            Month::April => "apr",
            Month::May => "may",
            Month::June => "jun",
            Month::July => "jul",
            Month::August => "aug",
            Month::September => "sep",
            Month::October => "oct",
            Month::November => "nov",
            Month::December => "dec",
        },
    }
}
