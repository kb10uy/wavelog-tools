use serde::{Deserialize, Deserializer};
use time::{Date, format_description::BorrowedFormatItem, macros::format_description};

use crate::wavelog::{WavelogClient, WavelogError};

const MAX_PER_PAGE: usize = 5000;
const QUERY_DATE: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QsoQuery {
    pub station_ids: Vec<u64>,
    pub qso_since: Option<Date>,
    pub qso_until: Option<Date>,
}

impl QsoQuery {
    fn to_query_pairs(&self, since_id: u64) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("format", "adif".to_string()),
            ("per_page", MAX_PER_PAGE.to_string()),
            ("since_id", since_id.to_string()),
        ];
        if !self.station_ids.is_empty() {
            let ids: Vec<_> = self.station_ids.iter().map(|i| i.to_string()).collect();
            pairs.push(("station_id", ids.join(",")));
        }
        if let Some(date) = self.qso_since {
            pairs.push(("qso_since", format_date(date)));
        }
        if let Some(date) = self.qso_until {
            pairs.push(("qso_until", format_date(date)));
        }
        pairs
    }
}

fn format_date(date: Date) -> String {
    date.format(QUERY_DATE).expect("date must be formattable")
}

impl WavelogClient {
    pub fn qso_adif_pages<'a>(&'a self, query: &'a QsoQuery) -> QsoAdifPages<'a> {
        QsoAdifPages {
            client: self,
            query,
            since_id: 0,
            finished: false,
        }
    }
}

#[derive(Debug)]
pub struct QsoAdifPages<'a> {
    client: &'a WavelogClient,
    query: &'a QsoQuery,
    since_id: u64,
    finished: bool,
}

impl QsoAdifPages<'_> {
    fn fetch_next(&mut self) -> Result<Option<String>, WavelogError> {
        let pairs = self.query.to_query_pairs(self.since_id);
        let response: QsoAdifResponse = self.client.get("qso", &pairs)?;

        let Some(adif) = response.data.adif.filter(|_| response.data.exported > 0) else {
            self.finished = true;
            return Ok(None);
        };
        if response.data.lastfetchedid <= self.since_id {
            self.finished = true;
        } else {
            self.since_id = response.data.lastfetchedid;
            self.finished = !response.meta.has_more;
        }
        Ok(Some(adif))
    }
}

impl Iterator for QsoAdifPages<'_> {
    type Item = Result<String, WavelogError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        match self.fetch_next() {
            Ok(page) => page.map(Ok),
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct QsoAdifResponse {
    data: QsoAdifData,
    meta: QsoAdifMeta,
}

#[derive(Debug, Deserialize)]
struct QsoAdifData {
    #[serde(deserialize_with = "deserialize_lenient_u64")]
    exported: u64,

    #[serde(default, deserialize_with = "deserialize_lenient_u64")]
    lastfetchedid: u64,

    adif: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QsoAdifMeta {
    #[serde(default)]
    has_more: bool,
}

fn deserialize_lenient_u64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Lenient {
        Number(u64),
        Text(String),
        Null(()),
    }

    match Lenient::deserialize(deserializer)? {
        Lenient::Number(n) => Ok(n),
        Lenient::Text(s) => s.parse().map_err(serde::de::Error::custom),
        Lenient::Null(()) => Ok(0),
    }
}
