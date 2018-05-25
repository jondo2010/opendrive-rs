pub mod flexible_boolean {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", value);
        serializer.serialize_str(&s)
    }

    /// Custom deserializer that can accept strings [0, 1, true, false]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::str::FromStr;
        let s: String = String::deserialize(deserializer)?;
        bool::from_str(&s).or_else(|_e| {
            i8::from_str(&s)
                .and_then(|x| Ok(x > 0))
                .map_err(serde::de::Error::custom)
        })
    }
}

pub mod odr_dateformat {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};
    const DATE_FORMAT: &'static str = "%a %b %d %H:%M:%S %Y";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(DATE_FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, DATE_FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

/// This module overrides the Derived Serialize and Deserialize traits on Angle
/// from the euclid crate in order to flatten the value
pub mod angle {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use types;

    pub fn serialize<S>(angle: &types::Angle, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", angle.radians);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<types::Angle, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(types::Angle::radians(try!(Deserialize::deserialize(deserializer))))
    }
}
