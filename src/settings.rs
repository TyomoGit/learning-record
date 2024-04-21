use chrono::{NaiveTime, Weekday};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub start: Start,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct Start {
    pub weekday: Weekday,
    pub time: NaiveTime,
}
