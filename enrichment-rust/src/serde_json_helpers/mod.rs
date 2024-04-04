use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

pub fn deserialize_f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse().map_err(de::Error::custom)?,
        Value::Number(num) => num.as_f64().ok_or(de::Error::custom("Invalid number"))?,
        _ => return Err(de::Error::custom("wrong type"))
    })
}

pub fn serialize_f64_2_decimals<S: serde::Serializer>(f: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_f64((f * 100.0).round() / 100.0)
}

pub fn serialize_u64_optional_none_as_minus_one<S: serde::Serializer>(f: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error> {
    match f {
        Some(f) => serializer.serialize_u64(*f),
        None => serializer.serialize_i64(-1),
    }
}

pub fn merge_jsons(base: &mut Value, overwrite: Value) {
    match (base, overwrite) {
        (a @ &mut Value::Object(_), Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge_jsons(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

pub mod ymd_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn serialize<S>(
        date: &NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(dt)
    }
}

pub mod ymd_date_format_optional {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn serialize<S>(
        date: &Option<NaiveDate>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => {
                let s = format!("{}", date.format(FORMAT));
                serializer.serialize_str(&s)
            }
            None => serializer.serialize_str(""),
            
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "" => Ok(None),
            s => {
                let dt = NaiveDate::parse_from_str(s, FORMAT).map_err(serde::de::Error::custom)?;
                Ok(Some(dt))
            }
        }
    }
}
