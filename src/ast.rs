use chrono::{NaiveDate, NaiveTime, TimeDelta};

use crate::settings::Settings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub settings: Option<Settings>,
    pub records: Vec<DayRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayRecord {
    pub date: NaiveDate,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub tags: Option<Tags>,
    pub info: Vec<EventInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tags {
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub title: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventInfo {
    pub time: NaiveTime,
    pub duration: TimeDelta,
}
