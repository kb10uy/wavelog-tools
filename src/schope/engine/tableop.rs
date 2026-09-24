use mlua::prelude::*;

const SCHOPE_TYPE_KEY: &str = "__schope_type";
const SCHOPE_TYPE_LIST: &str = "list";
const SCHOPE_TYPE_MAP: &str = "map";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TableType {
    List,
    Map,
}

pub fn ensure_list(lua: &Lua, table: LuaTable) -> LuaResult<LuaTable> {
    match table.metatable() {
        Some(mt) => {
            mt.set(SCHOPE_TYPE_KEY, SCHOPE_TYPE_LIST)?;
        }
        None => {
            let mt = lua.create_table()?;
            mt.set(SCHOPE_TYPE_KEY, SCHOPE_TYPE_LIST)?;
            table.set_metatable(Some(mt))?;
        }
    }
    Ok(table)
}

pub fn ensure_map(lua: &Lua, table: LuaTable) -> LuaResult<LuaTable> {
    match table.metatable() {
        Some(mt) => {
            mt.set(SCHOPE_TYPE_KEY, SCHOPE_TYPE_MAP)?;
        }
        None => {
            let mt = lua.create_table()?;
            mt.set(SCHOPE_TYPE_KEY, SCHOPE_TYPE_MAP)?;
            table.set_metatable(Some(mt))?;
        }
    }
    Ok(table)
}

pub fn check_table_type(table: &LuaTable) -> LuaResult<TableType> {
    if let Some(mt) = table.metatable() {
        let ensured_type: Option<String> = mt.get(SCHOPE_TYPE_KEY)?;
        match ensured_type.as_deref() {
            Some(SCHOPE_TYPE_LIST) => return Ok(TableType::List),
            Some(SCHOPE_TYPE_MAP) => return Ok(TableType::Map),
            _ => (),
        }
    }

    match table.raw_len() {
        0 => Ok(TableType::Map),
        n => {
            for i in 1..=n {
                if !table.contains_key(i)? {
                    return Ok(TableType::Map);
                }
            }
            Ok(TableType::List)
        }
    }
}
