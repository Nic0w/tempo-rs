use std::{collections::HashMap, fmt};

use chrono::{DateTime, Utc};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

/// API's main output struct.
#[derive(Debug, Deserialize)]
pub struct TempoCalendars {
    /// Contains the calendars (sets of days) for the requested periods.
    #[serde(deserialize_with = "vec_or_struct")]
    pub tempo_like_calendars: Vec<Calendar>,
}

impl TempoCalendars {
    /// When requesting next-day color, this function is a short-hand to directly unwrap next-day data from the nested struct.
    pub fn unwrap_first_day_value(&self) -> Option<&CalendarValue> {
        self.tempo_like_calendars
            .first()
            .and_then(|cal| cal.values.first())
    }

    /// When requesting historical data, this function is a short-hand to iterate over values despite the nested structure.
    pub fn unwrap_days_values(&self) -> impl Iterator<Item = &CalendarValue> {
        self.tempo_like_calendars
            .iter()
            .flat_map(|calendar| calendar.values.iter())
    }
}

/// Contains a set of days.
/// Server returns data sorted from closest to farthest date relative to the `start_date` date,
/// meaning that data is sorted from most recent to most ancient date.
#[derive(Debug, Deserialize)]
pub struct Calendar {
    #[serde(with = "rte_api_date")]
    pub start_date: DateTime<Utc>,
    #[serde(with = "rte_api_date")]
    pub end_date: DateTime<Utc>,

    pub values: Vec<CalendarValue>,
}

/// A "Tempo" period.
/// Despite that
///  - a Tempo day is split in two periods (peak, off peak),
///  - a Tempo day runs from 6AM to 6AM next-day,
///
/// this is always (?) for an unknown reason a full 24h period from midnight to midnight.
/// It kinda makes it easier to reason about what color a day is, though if one wants to be precise about which price to apply depending on the hour of the day, it is necessary to do that calculation again.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct CalendarValue {
    ///Start of the day long period.
    #[serde(with = "rte_api_date")]
    pub start_date: DateTime<Utc>,

    ///End of the day long period.
    #[serde(with = "rte_api_date")]
    pub end_date: DateTime<Utc>,

    ///Date/time at which the period was last updated.
    #[serde(with = "rte_api_date")]
    pub updated_date: DateTime<Utc>,

    ///Color of the day.
    pub value: TempoColor,

    /// ???
    pub fallback: Option<bool>,
}

///Tempo day color.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TempoColor {
    /// Blue day
    Blue,

    /// White day
    White,

    /// Red day
    Red,
}

impl fmt::Display for TempoColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TempoColor::Blue => write!(f, "blue"),
            TempoColor::White => write!(f, "white"),
            TempoColor::Red => write!(f, "red"),
        }
    }
}

fn vec_or_struct<'de, D>(deserializer: D) -> Result<Vec<Calendar>, D::Error>
where
    D: Deserializer<'de>,
{
    struct VecOrStruct;

    impl<'de> Visitor<'de> for VecOrStruct {
        type Value = Vec<Calendar>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("vec or map")
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let value: Calendar =
                Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;

            Ok(vec![value])
        }
    }

    deserializer.deserialize_any(VecOrStruct)
}

mod rte_api_date {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%FT%T%:z";

    #[allow(dead_code)]
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Error {
    pub error: String,
    pub error_description: String,
    pub error_uri: String,
    pub error_details: HashMap<String, String>,
}
