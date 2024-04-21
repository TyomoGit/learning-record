use chrono::{DateTime, Datelike, Local, NaiveDateTime, NaiveTime};

use crate::ast::{self, EventInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotPast(Vec<EventInfo>),
}

pub fn calc_weekly_records(file: &ast::File, today: DateTime<Local>) -> Result<NaiveTime, Error> {
    let (start_weekday, start_time) = match &file.settings {
        Some(settings) => (settings.start.weekday, settings.start.time),
        None => (
            (today.naive_local() - chrono::Duration::days(7)).weekday(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        ),
    };
    let start_date = {
        let mut date = today;

        if today.weekday() == start_weekday {
            if today.time() < start_time {
                today - chrono::Duration::days(7)
            } else {
                today
            }
        } else {
            while date.weekday() != start_weekday {
                date -= chrono::Duration::days(1);
            }

            date
        }
    }
    .with_time(start_time)
    .unwrap()
    .naive_local();

    dbg!(start_weekday, start_date, start_time);

    let mut sum: NaiveTime = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    for day_record in &file.records {
        if day_record.date < start_date.date() {
            continue;
        }
        for event in &day_record.events {
            for event_info in &event.info {
                let event_datetime = NaiveDateTime::new(day_record.date, event_info.time);
                if event_datetime < start_date && event_datetime < today.naive_local() {
                    continue;
                }

                sum += event_info.duration;
            }
        }
    }

    Ok(sum)
}
