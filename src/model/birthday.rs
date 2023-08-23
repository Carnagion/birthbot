//! Core birthday-related types, including parsing and display implementations.

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

use snafu::prelude::*;

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

    /// Parses a "human readable" date, following the British convention of `<day> <month> <year>`.
    /// You can also optionally include the 24-hour time and the timezone offset, separated by commas.
    ///
    /// These are some examples of some valid human readable dates:
    /// - `1 November 2007`
    /// - `19 July 2002, 01:13`
    /// - `23 June 1996, 14:35, +09:00`
    ///
    /// # Errors
    ///
    /// If the input string is empty, [`ParseBirthdayError
    /// ::Empty`] will be returned.
    /// Otherwise, [`ParseBirthdayError
    /// ::Invalid`] will be returned if the input string isn't valid.
    fn parse_human_date(string: &str) -> Result<Self, ParseBirthdayError> {
        let mut split = string.splitn(3, ',');
        let (Some(date), time, timezone) = (split.next(), split.next(), split.next()) else {
            return Err(ParseBirthdayError::Empty)
        };

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

    /// Parses an [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339) date, with optional time and timezone offset components.
    ///
    /// These are some examples of valid dates:
    /// - `2007-11-01`
    /// - `2002-07-19T01:13`
    /// - `1996-06-23T14:35+09:00`
    /// - `2017-10-27T00:56Z`
    ///
    /// # Errors
    ///
    /// If the input string is empty, [`ParseBirthdayError
    /// ::Empty`] will be returned.
    /// Otherwise, [`ParseBirthdayError
    /// ::Invalid`] will be returned if the input string isn't valid.
    fn parse_rfc3339_date(string: &str) -> Result<Self, ParseBirthdayError> {
        let string = string.trim();

        if string.is_empty() {
            return Err(ParseBirthdayError::Empty);
        }

        // Attempt to parse the date, time, and timezone offset.
        // Otherwise, just parse the date, and (optionally) the time.
        if let Ok(datetime) = DateTime::parse_from_rfc3339(string) {
            return Ok(Self(datetime));
        }

        let (date, remainder) = NaiveDate::parse_and_remainder(string, "%Y-%m-%d")?;
        let time = if remainder.is_empty() {
            // Default time of midnight i.e. 00:00
            NaiveTime::default()
        } else {
            NaiveTime::parse_from_str(remainder, "T%H:%M:%S")?
        };

        let offset: FixedOffset = FixedOffset::east_opt(0).unwrap(); // PANICS: A zero timezone offset will always be valid
        let datetime = DateTime::from_utc(NaiveDateTime::new(date, time), offset);
        Ok(Self(datetime))
    }
}

impl Display for Birthday {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        Self::format(&self.0).fmt(formatter)
    }
}

/// Possible errors that can arise while parsing a [`Birthday`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Snafu)]
pub enum ParseBirthdayError {
    /// The input was empty.
    #[snafu(display("input is empty"))]
    Empty,
    /// The input was formatted incorrectly.
    #[snafu(context(false), display("{}.", source))]
    Invalid {
        /// The underlying source of the parsing error.
        source: ParseError,
    },
}

/// A combination of errors that can arise when trying multiple strategies of parsing a [`Birthday`].
#[derive(Clone, Debug, Eq, Hash, PartialEq, Snafu)]
#[snafu(display("Invalid birthday format ({}).
Valid formats include RFC-3339 (such as `2007-11-01`, `2002-07-19T01:13`, or `1996-06-23T14:35+09:00`) and day-month-year (such as `1 November 2007`, `19 July 2002, 01:13`, or `23 June 1996, 14:35, +09:00`).",
rest.iter().map(|err| err.to_string()).fold(first.to_string(), |mut string, err| {
    string.push_str(" or ");
    string.push_str(&err);
    string
})))]
pub struct ParseCombinedBirthdayError {
    first: ParseBirthdayError,
    rest: Vec<ParseBirthdayError>,
}

impl FromStr for Birthday {
    type Err = ParseCombinedBirthdayError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // Attempts a human readable date first, then falls back to an RFC 3339 date.
        Self::parse_human_date(string)
            .map_err(|human_err| ParseCombinedBirthdayError {
                first: human_err,
                rest: vec![],
            })
            .or_else(|mut err| {
                Self::parse_rfc3339_date(string).map_err(|rfc_err| {
                    err.rest.push(rfc_err);
                    err
                })
            })
    }
}
