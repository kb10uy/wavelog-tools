use callfind::grid_locator::GridLocator;
use compact_str::CompactString;
use mlua::prelude::*;

use crate::schope::data::{exchange::Exchange, record::Record};

#[derive(Debug, Clone, PartialEq)]
pub struct QslCardEntry {
    pub qso: Record,
    pub exchange: Exchange,
    pub info: QslInfo,
}

impl IntoLua for QslCardEntry {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("qso", self.qso)?;
        table.set("exchange", self.exchange)?;
        table.set("info", self.info)?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QslInfo {
    pub station: QslStation,
    pub operator: QslOperator,
    pub instrument: QslInstrument,
    pub card: QslCard,
}

impl IntoLua for QslInfo {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("station", self.station)?;
        table.set("operator", self.operator)?;
        table.set("instrument", self.instrument)?;
        table.set("card", self.card)?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslStation {
    pub callsign: Option<CompactString>,
    pub grid: Option<GridLocator>,

    pub city: Option<CompactString>,
    pub state: Option<CompactString>,
    pub state_name: Option<CompactString>,
    pub country: Option<CompactString>,

    pub pota: Option<CompactString>,
    pub sota: Option<CompactString>,
    pub wwff: Option<CompactString>,
    pub iota: Option<CompactString>,

    pub sig: Option<CompactString>,
    pub sig_info: Option<CompactString>,

    pub parks: Vec<QslPark>,
}

impl IntoLua for QslStation {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("callsign", self.callsign.as_deref())?;
        table.set("grid", self.grid.map(|g| g.to_string()))?;
        table.set("city", self.city.as_deref())?;
        table.set("state", self.state.as_deref())?;
        table.set("state_name", self.state_name.as_deref())?;
        table.set("country", self.country.as_deref())?;
        table.set("pota", self.pota.as_deref())?;
        table.set("sota", self.sota.as_deref())?;
        table.set("wwff", self.wwff.as_deref())?;
        table.set("iota", self.iota.as_deref())?;
        table.set("sig", self.sig.as_deref())?;
        table.set("sig_info", self.sig_info.as_deref())?;
        table.set("parks", self.parks)?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslPark {
    pub reference: CompactString,
    pub location: Option<CompactString>,
    pub name_en: Option<CompactString>,
    pub name_ja: Option<CompactString>,
}

impl IntoLua for QslPark {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("reference", self.reference.as_str())?;
        table.set("location", self.location.as_deref())?;
        table.set("name_en", self.name_en.as_deref())?;
        table.set("name_ja", self.name_ja.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslOperator {
    pub callsign: Option<CompactString>,
    pub name: Option<CompactString>,
}

impl IntoLua for QslOperator {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("callsign", self.callsign.as_deref())?;
        table.set("name", self.name.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QslInstrument {
    pub antenna: Option<CompactString>,
    pub rig: Option<CompactString>,
    pub power: Option<f64>,
}

impl IntoLua for QslInstrument {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("antenna", self.antenna.as_deref())?;
        table.set("rig", self.rig.as_deref())?;
        table.set("power", self.power)?;

        Ok(LuaValue::Table(table))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QslCard {
    pub should_send: bool,
    pub received: bool,
    pub manager: Option<CompactString>,
}

impl IntoLua for QslCard {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("should_send", self.should_send)?;
        table.set("received", self.received)?;
        table.set("manager", self.manager.as_deref())?;

        Ok(LuaValue::Table(table))
    }
}
