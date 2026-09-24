use adif_reader::document::Record;
use compact_str::{CompactString, ToCompactString};

use crate::qso::get_optional_field_oneof;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QsoExchange {
    pub report: Option<CompactString>,
    pub number: Option<CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QsoExchanges {
    pub sent: QsoExchange,
    pub received: QsoExchange,
}

impl QsoExchanges {
    pub fn new(record: &Record) -> QsoExchanges {
        let sent_report = get_optional_field_oneof(record, &["RST_SENT"]);
        let sent_number = get_optional_field_oneof(record, &["STX", "STX_STRING"]);
        let received_report = get_optional_field_oneof(record, &["RST_RCVD"]);
        let received_number = get_optional_field_oneof(record, &["SRX", "SRX_STRING"]);

        let sent = QsoExchange {
            report: sent_report.map(|s| s.to_compact_string()),
            number: sent_number.map(|s| s.to_compact_string()),
        };
        let received = QsoExchange {
            report: received_report.map(|s| s.to_compact_string()),
            number: received_number.map(|s| s.to_compact_string()),
        };
        QsoExchanges { sent, received }
    }
}
