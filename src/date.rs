use time::{Date, OffsetDateTime};

pub fn now() -> Date {
    OffsetDateTime::now_local()
        .inspect_err(|e| eprintln!("{e}"))
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .date()
}
