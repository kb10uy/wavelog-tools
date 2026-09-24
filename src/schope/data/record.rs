use compact_str::{CompactString, ToCompactString};
use mlua::prelude::*;
use time::OffsetDateTime;

use crate::{qso::record::QsoRecord, schope::library::datetime::SchopeDateTime};

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub datetime: SchopeDateTime,
    pub band: CompactString,
    pub freq: CompactString,
    pub mode: CompactString,
    pub call: CompactString,
}

impl IntoLua for Record {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("datetime", self.datetime)?;
        table.set("band", self.band.to_string())?;
        table.set(
            "freq",
            self.freq.parse::<f64>().map_err(LuaError::external)?,
        )?;
        table.set("freq_str", self.freq.to_string())?;
        table.set("mode", self.mode.to_string())?;
        table.set("call", self.call.to_string())?;

        Ok(LuaValue::Table(table))
    }
}

impl From<QsoRecord> for Record {
    fn from(value: QsoRecord) -> Self {
        Record {
            datetime: OffsetDateTime::from(value.datetime).into(),
            band: value.band.to_compact_string(),
            freq: value.frequency,
            mode: value.mode,
            call: value.call,
        }
    }
}
