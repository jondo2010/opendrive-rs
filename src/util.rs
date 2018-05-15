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
