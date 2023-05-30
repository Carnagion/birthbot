use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use chrono::{
    format::{self, Parsed, StrftimeItems},
    prelude::*,
    ParseError,
};

use serde::{Deserialize, Serialize};

use snafu::Snafu;

/// A birthday represented as a date and time with a specific timezone.
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Birthday(pub DateTime<FixedOffset>);

impl Birthday {
    const DISPLAY_FORMAT: &str = "%d %B %Y %:z";

    /// Formats a [`DateTime`] in the same way as a [`Birthday`].
    pub fn format<Tz>(datetime: &DateTime<Tz>) -> impl Display
    where
        Tz: TimeZone,
        Tz::Offset: Display,
    {
        datetime.format(Self::DISPLAY_FORMAT)
    }

    fn parse_from_parts(
        date: &str,
        time: Option<&str>,
        timezone: Option<&str>,
    ) -> Result<Self, ParseError> {
        let mut parsed = Parsed::new();

        format::parse(&mut parsed, date.trim(), StrftimeItems::new("%d %B %Y"))?;
        match time {
            None => {
                parsed.set_hour(0)?;
                parsed.set_minute(0)?;
                parsed.set_second(0)
            },
            Some(time) => format::parse(&mut parsed, time.trim(), StrftimeItems::new("%H:%M:%S")),
        }?;
        match timezone {
            None => parsed.set_offset(0),
            Some(timezone) => {
                format::parse(&mut parsed, timezone.trim(), StrftimeItems::new("%:z"))
            },
        }?;

        Ok(Self(parsed.to_datetime()?))
    }
}

impl Display for Birthday {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        Self::format(&self.0).fmt(formatter)
    }
}

/// Possible errors that can arise while parsing a [`Birthday`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Snafu)]
pub enum BirthdayParseError {
    /// The input was empty.
    #[snafu(display("Input is empty"))]
    Empty,
    /// The input was formatted incorrectly.
    #[snafu(context(false), display("{}", source))]
    Invalid {
        /// The underlying source of the parsing error.
        source: ParseError,
    },
}

impl FromStr for Birthday {
    type Err = BirthdayParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut split = string.splitn(3, ',');
        let birthday = match (split.next(), split.next(), split.next()) {
            (Some(date), time, timezone) => Self::parse_from_parts(date, time, timezone)?,
            _ => Err(BirthdayParseError::Empty)?,
        };
        Ok(birthday)
    }
}
