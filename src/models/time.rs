use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Asia::Kolkata;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Custom time format used in all responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Time {
    inner: Option<DateTime<Utc>>,
}

impl Time {
    /// List of known time formats without timezone
    const LAYOUTS: &'static [&'static str] = &["%Y-%m-%d", "%Y-%m-%d %H:%M:%S"];

    /// List of known time formats with timezone
    const ZONED_LAYOUTS: &'static [&'static str] = &[
        "%Y-%m-%dT%H:%M:%S%z",
        "%Y-%m-%dT%H:%M:%S%.f%:z", // RFC3339-like
    ];

    /// Create a new Time instance
    pub fn new(dt: DateTime<Utc>) -> Self {
        Time { inner: Some(dt) }
    }

    /// Create an empty/null Time instance
    pub fn null() -> Self {
        Time { inner: None }
    }

    /// Create from Unix timestamp
    pub fn from_timestamp(timestamp: i64) -> Self {
        if timestamp == 0 {
            Self::null()
        } else {
            match DateTime::from_timestamp(timestamp, 0) {
                Some(dt) => Time { inner: Some(dt) },
                None => Time { inner: None },
            }
        }
    }

    /// Check if the time is null/empty
    pub fn is_null(&self) -> bool {
        self.inner.is_none()
    }

    /// Get the inner DateTime if present
    pub fn as_datetime(&self) -> Option<DateTime<Utc>> {
        self.inner
    }

    /// Parse time from string
    fn parse_time(s: &str) -> Result<Option<DateTime<Utc>>, String> {
        let s = s.trim();

        // Handle empty or null strings
        if s.is_empty() || s == "null" {
            return Ok(None);
        }

        // Try parsing with zoneless layouts (assuming IST/Kolkata timezone)
        for layout in Self::LAYOUTS {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s, layout) {
                // Convert to IST then to UTC
                if let Some(ist_dt) = Kolkata.from_local_datetime(&naive_dt).single() {
                    return Ok(Some(ist_dt.with_timezone(&Utc)));
                }
            }
            // Also try parsing as date only
            if let Ok(naive_date) = NaiveDate::parse_from_str(s, layout) {
                let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
                if let Some(ist_dt) = Kolkata.from_local_datetime(&naive_dt).single() {
                    return Ok(Some(ist_dt.with_timezone(&Utc)));
                }
            }
        }

        // Try parsing with zoned layouts
        for layout in Self::ZONED_LAYOUTS {
            if let Ok(dt) = DateTime::parse_from_str(s, layout) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }
        }

        // Try parsing RFC3339 directly
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Ok(Some(dt.with_timezone(&Utc)));
        }

        Err("unknown time format".to_string())
    }
}

// Implement Serialize for Time
impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.inner {
            Some(dt) => {
                // Serialize as RFC3339 string
                serializer.serialize_str(&dt.to_rfc3339())
            }
            None => serializer.serialize_none(),
        }
    }
}

// Implement Deserialize for Time
impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Time, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;

        match s {
            Some(s) => {
                let s = s.trim().trim_matches('"');
                Self::parse_time(s)
                    .map(|opt_dt| Time { inner: opt_dt })
                    .map_err(serde::de::Error::custom)
            }
            None => Ok(Time { inner: None }),
        }
    }
}

// Optional: Implement Display for Time
impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            Some(dt) => write!(f, "{}", dt.to_rfc3339()),
            None => write!(f, "null"),
        }
    }
}

// Optional: Implement Default for Time
impl Default for Time {
    fn default() -> Self {
        Time { inner: None }
    }
}

// Optional: Conversion traits
impl From<DateTime<Utc>> for Time {
    fn from(dt: DateTime<Utc>) -> Self {
        Time { inner: Some(dt) }
    }
}

impl From<Option<DateTime<Utc>>> for Time {
    fn from(opt_dt: Option<DateTime<Utc>>) -> Self {
        Time { inner: opt_dt }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_only() {
        let result = Time::parse_time("2024-01-15").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_datetime() {
        let result = Time::parse_time("2024-01-15 14:30:00").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_rfc3339() {
        let result = Time::parse_time("2024-01-15T14:30:00+05:30").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_null() {
        let result = Time::parse_time("null").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_empty() {
        let result = Time::parse_time("").unwrap();
        assert!(result.is_none());
    }
}
