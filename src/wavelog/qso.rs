use clap::ValueEnum;
use serde::{Deserialize, Deserializer};
use time::{Date, format_description::BorrowedFormatItem, macros::format_description};

use crate::wavelog::{WavelogClient, WavelogError};

const MAX_PER_PAGE: usize = 5000;
const QUERY_DATE: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QsoQuery {
    pub station_ids: Vec<u64>,
    pub callsign: Option<String>,
    pub band: Option<String>,
    pub mode: Option<String>,
    pub qsl_filter: Vec<QslFilter>,
    pub since_id: u64,
    pub qso_since: Option<Date>,
    pub qso_until: Option<Date>,
}

/// Confirmation type matched by `qsl_filter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum QslFilter {
    /// Confirmed via LoTW.
    Lotw,

    /// Confirmed via paper QSL card.
    Qsl,

    /// Confirmed via eQSL.
    Eqsl,

    /// Confirmed via QRZ.com.
    Qrz,

    /// Confirmed via Club Log.
    Clublog,
}

impl QslFilter {
    fn as_str(self) -> &'static str {
        match self {
            QslFilter::Lotw => "lotw",
            QslFilter::Qsl => "qsl",
            QslFilter::Eqsl => "eqsl",
            QslFilter::Qrz => "qrz",
            QslFilter::Clublog => "clublog",
        }
    }
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
        if let Some(callsign) = &self.callsign {
            pairs.push(("callsign", callsign.clone()));
        }
        if let Some(band) = &self.band {
            pairs.push(("band", band.clone()));
        }
        if let Some(mode) = &self.mode {
            pairs.push(("mode", mode.clone()));
        }
        if !self.qsl_filter.is_empty() {
            let types: Vec<_> = self.qsl_filter.iter().map(|f| f.as_str()).collect();
            pairs.push(("qsl_filter", types.join(",")));
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
            since_id: query.since_id,
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

/// ADIF text of a page fetched from Wavelog, including its header.
#[derive(Debug, Clone)]
pub struct QsoAdifPage {
    pub exported: u64,
    pub adif: String,
}

impl QsoAdifPages<'_> {
    fn fetch_next(&mut self) -> Result<Option<QsoAdifPage>, WavelogError> {
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
        Ok(Some(QsoAdifPage {
            exported: response.data.exported,
            adif,
        }))
    }
}

impl Iterator for QsoAdifPages<'_> {
    type Item = Result<QsoAdifPage, WavelogError>;

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

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn builds_query_pairs() {
        let query = QsoQuery {
            station_ids: vec![1, 2],
            callsign: Some("JA1ZLO".to_string()),
            band: Some("20m".to_string()),
            mode: Some("FT8".to_string()),
            qsl_filter: vec![QslFilter::Lotw, QslFilter::Qsl],
            since_id: 10,
            qso_since: Some(date!(2026 - 01 - 01)),
            qso_until: Some(date!(2026 - 12 - 31)),
        };
        let pairs = query.to_query_pairs(42);
        let get = |key: &str| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(get("since_id"), Some("42"));
        assert_eq!(get("station_id"), Some("1,2"));
        assert_eq!(get("callsign"), Some("JA1ZLO"));
        assert_eq!(get("band"), Some("20m"));
        assert_eq!(get("mode"), Some("FT8"));
        assert_eq!(get("qsl_filter"), Some("lotw,qsl"));
        assert_eq!(get("qso_since"), Some("2026-01-01"));
        assert_eq!(get("qso_until"), Some("2026-12-31"));
    }

    #[test]
    fn omits_empty_filters() {
        let pairs = QsoQuery::default().to_query_pairs(0);
        let keys: Vec<_> = pairs.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, ["format", "per_page", "since_id"]);
    }
}
