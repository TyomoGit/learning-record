use chrono::{NaiveDate, NaiveTime, TimeDelta};

use crate::{
    ast::{DayRecord, Event, EventInfo, File, Tag, Tags},
    settings::Settings,
};

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    ExpectedChars { expected: Vec<char>, found: char },
    UnexpectedEof,
    InvalidDate,
    InvalidDurationFormat,
    TomlError(toml::de::Error),
}

#[derive(Debug, Clone)]
pub struct Parser {
    source: Vec<char>,

    start: usize,
    current: usize,

    line: usize,
    column: usize,
}

impl Parser {
    pub fn new(source: Vec<char>) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn parse_file(&mut self) -> Result<File> {
        self.skip_space();
        let settings = if self.peek() == Some('-') {
            Some(self.parse_settings()?)
        } else {
            None
        };

        while matches!(self.peek(), Some('\n' | '\r')) {
            self.advance();
        }
        self.clear();

        let mut records: Vec<DayRecord> = Vec::new();
        while self.peek().is_some() {
            records.push(self.parse_day_record()?);

            while matches!(self.peek(), Some('\n' | '\r')) {
                self.advance();
            }
            self.clear();
        }

        Ok(File { records, settings })
    }

    fn parse_settings(&mut self) -> Result<Settings> {
        self.expect_string("---\n")?;
        self.clear();
        while self.peek().is_some() {
            self.extract_until('\n');
            self.expect_char('\n')?;
            if self.peek() == Some('-') {
                let Some(toml) = self.collect() else {
                    return Err(self.make_error(ParseErrorKind::UnexpectedEof));
                };
                self.expect_string("---\n")?;
                let settings: Settings = match toml::from_str(&toml) {
                    Ok(settings) => settings,
                    Err(e) => return Err(self.make_error(ParseErrorKind::TomlError(e))),
                };
                return Ok(settings);
            }
        }

        Err(self.make_error(ParseErrorKind::UnexpectedEof))
    }

    fn parse_day_record(&mut self) -> Result<DayRecord> {
        let date = self.parse_date()?;

        self.skip_space();
        self.expect_char('\n')?;
        self.clear();
        let mut events = Vec::new();

        while let Some(c) = self.peek() {
            if c == '\n' {
                self.advance();
                self.clear();
                break;
            } else {
                events.push(self.parse_event()?);
            }
        }

        Ok(DayRecord { date, events })
    }

    fn parse_date(&mut self) -> Result<NaiveDate> {
        let year: i32 = self.extract_num()?.parse().expect("failed to parse year");
        self.expect_char('-')?;
        self.clear();

        let month: u32 = self.extract_num()?.parse().expect("failed to parse month");
        self.expect_char('-')?;
        self.clear();

        let day: u32 = self.extract_num()?.parse().expect("failed to parse day");
        self.clear();

        let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
            return Err(self.make_error(ParseErrorKind::InvalidDate));
        };

        Ok(date)
    }

    fn parse_event(&mut self) -> Result<Event> {
        let tags = if Some('[') == self.peek() {
            let tags = self.parse_tags()?;
            Some(tags)
        } else {
            None
        };

        let mut info = Vec::new();
        while let Some(c) = self.peek() {
            if c == '\n' {
                self.advance();
                self.clear();
                break;
            }

            self.skip_space();
            info.push(self.parse_event_info()?);
            self.skip_space();
            if Some(',') == self.peek() {
                self.advance();
                self.clear();
            }
        }

        Ok(Event { tags, info })
    }

    fn parse_tags(&mut self) -> Result<Tags> {
        self.expect_char('[')?;
        self.clear();
        let mut tags = Vec::new();

        while matches!(self.peek(), Some(c) if c != ']') {
            tags.push(self.parse_tag()?);
        }

        self.skip_space();

        self.expect_char(']')?;
        self.clear();

        Ok(Tags { tags })
    }

    fn parse_tag(&mut self) -> Result<Tag> {
        self.skip_space();
        while let Some(c) = self.peek() {
            if c.is_whitespace() || c == ']' || c == '(' {
                break;
            }

            self.advance();
        }

        let Some(tag) = self.collect() else {
            return Err(self.make_error(ParseErrorKind::UnexpectedEof));
        };

        let detail = if self.peek() == Some('(') {
            self.advance();
            self.clear();
            while let Some(c) = self.peek() {
                if c == ')' {
                    break;
                }
                self.advance();
            }

            let result = self.collect();

            self.advance();
            self.clear();
            result
        } else {
            None
        };

        Ok(Tag { title: tag, detail })
    }

    fn parse_event_info(&mut self) -> Result<EventInfo> {
        // time
        let Ok(date_hours) = self.extract_num()?.parse() else {
            return Err(self.make_error(ParseErrorKind::UnexpectedEof));
        };

        self.expect_char(':')?;
        self.clear();

        let Ok(date_minutes) = self.extract_num()?.parse() else {
            return Err(self.make_error(ParseErrorKind::UnexpectedEof));
        };

        self.skip_space();
        self.expect_char('-')?;
        self.skip_space();

        // duration
        let mut hms: [Option<i64>; 3] = [None, None, None];
        let mut i: usize = 0;

        while i < hms.len() {
            let Ok(num_str) = self.extract_num() else {
                break;
            };

            let Ok(num) = num_str.parse() else {
                break;
            };

            let Some(unit) = self.advance() else {
                return Err(self.make_error(ParseErrorKind::UnexpectedEof));
            };

            let n_unit = match unit {
                'h' => 0,
                'm' => 1,
                's' => 2,
                _ => return Err(self.make_error(ParseErrorKind::InvalidDurationFormat)),
            };

            self.clear();

            if n_unit < i {
                return Err(self.make_error(ParseErrorKind::InvalidDurationFormat));
            }

            hms[n_unit] = Some(num);
            i = n_unit + 1;
        }

        if hms.iter().all(|num| num.is_none()) {
            return Err(self.make_error(ParseErrorKind::InvalidDurationFormat));
        }

        let Some(time) = NaiveTime::from_hms_opt(date_hours, date_minutes, 0) else {
            return Err(self.make_error(ParseErrorKind::InvalidDurationFormat));
        };

        let duration = TimeDelta::hours(hms.first().unwrap().unwrap_or_default())
            + TimeDelta::minutes(hms.get(1).unwrap().unwrap_or_default())
            + TimeDelta::seconds(hms.get(2).unwrap().unwrap_or_default());

        Ok(EventInfo { time, duration })
    }

    #[must_use]
    fn make_error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(kind, self.line, self.column)
    }

    fn extract_num(&mut self) -> Result<String> {
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.advance();
        }

        match self.collect() {
            Some(num) => Ok(num),
            None => Err(self.make_error(ParseErrorKind::UnexpectedEof)),
        }
    }

    fn expect_string(&mut self, s: &str) -> Result<()> {
        s.chars()
            .map(|c| self.expect_char(c))
            .find(Result::is_err)
            .unwrap_or(Ok(()))
    }

    fn expect_char(&mut self, c: char) -> Result<()> {
        self.expect_chars(std::iter::once(c))
    }

    fn expect_chars(&mut self, chars: impl IntoIterator<Item = char> + Clone) -> Result<()> {
        if let Some(c) = self.peek() {
            if chars.clone().into_iter().any(|char| char == c) {
                self.advance();
                Ok(())
            } else {
                Err(self.make_error(ParseErrorKind::ExpectedChars {
                    expected: chars.into_iter().collect(),
                    found: c,
                }))
            }
        } else {
            Err(self.make_error(ParseErrorKind::UnexpectedEof))
        }
    }

    fn skip_space(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace() && c != '\n') {
            self.advance();
        }

        self.clear();
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.source.get(self.current);
        self.current += 1;

        if let Some(c) = c {
            if *c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        c.cloned()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.current).cloned()
    }

    fn extract_until(&mut self, c: char) {
        while let Some(current) = self.peek() {
            if current == c {
                break;
            }
            self.advance();
        }
    }

    fn collect(&mut self) -> Option<String> {
        let result = self
            .source
            .get(self.start..self.current)?
            .iter()
            .cloned()
            .collect::<String>()
            .into();

        self.clear();

        result
    }

    fn clear(&mut self) {
        self.start = self.current;
    }
}
