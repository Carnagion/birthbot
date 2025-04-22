use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, FixedOffset, Timelike};

use poise::ChoiceParameter;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Birthday(pub DateTime<FixedOffset>);

impl Display for Birthday {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let fmt = if self.0.hour() == 0 && self.0.minute() == 0 && self.0.second() == 0 {
            "%d %B %Y (UTC%:z)"
        } else {
            "%d %B %Y %H:%M:%S (UTC%:z)"
        };

        self.0.format(fmt).fmt(f)
    }
}

// NOTE: `chrono` has a `Month` enum similar to this, and although it impls `FromStr`, it does not impl
//       `poise::ChoiceParameter`. Using it as a command argument is therefore a sub-par experience and
//       justifies the need for this type.
#[derive(Debug, ChoiceParameter, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Month {
    #[name = "January"]
    Jan = 1,
    #[name = "February"]
    Feb = 2,
    #[name = "March"]
    Mar = 3,
    #[name = "April"]
    Apr = 4,
    #[name = "May"]
    May = 5,
    #[name = "June"]
    Jun = 6,
    #[name = "July"]
    Jul = 7,
    #[name = "August"]
    Aug = 8,
    #[name = "September"]
    Sep = 9,
    #[name = "October"]
    Oct = 10,
    #[name = "November"]
    Nov = 11,
    #[name = "December"]
    Dec = 12,
}
