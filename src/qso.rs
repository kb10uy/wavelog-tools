pub mod band;
pub mod error;
pub mod exchange;
pub mod qsl;
pub mod record;

use adif_reader::document::Record;

use crate::qso::error::QsoError;

fn get_required_field<'a>(record: &'a Record, field: &'static str) -> Result<&'a str, QsoError> {
    get_optional_field(record, field).ok_or(QsoError::MissingAdifField(field))
}

fn get_optional_field<'a>(record: &'a Record, field: &'static str) -> Option<&'a str> {
    record.field(field).filter(|v| !v.is_empty())
}

fn get_optional_field_oneof<'a>(record: &'a Record, fields: &[&'static str]) -> Option<&'a str> {
    assert!(!fields.is_empty(), "fields must be specified");
    fields.iter().find_map(|f| get_optional_field(record, f))
}
