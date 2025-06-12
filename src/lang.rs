use time::Month;

#[derive(Clone, Copy)]
pub enum Lang {
    #[expect(dead_code)]
    Ru,
    En,
}

pub fn month_short_name(month: Month, lang: Lang) -> &'static str {
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
