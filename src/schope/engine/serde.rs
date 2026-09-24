use mlua::prelude::*;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

use crate::schope::engine::tableop::{TableType, check_table_type};

pub fn lua_to_json(lua_value: LuaValue) -> LuaResult<JsonValue> {
    match lua_value {
        LuaNil => Ok(JsonValue::Null),
        LuaValue::Boolean(b) => Ok(JsonValue::Bool(b)),
        LuaValue::Integer(i) => Ok(JsonValue::Number(
            JsonNumber::from_i128(i as i128).expect("overflow"),
        )),
        LuaValue::Number(n) => Ok(JsonValue::Number(
            JsonNumber::from_f64(n).expect("overflow"),
        )),
        LuaValue::String(s) => Ok(JsonValue::String(s.to_str()?.to_string())),
        LuaValue::Table(table) => match check_table_type(&table)? {
            TableType::List => {
                let values: Result<Vec<_>, _> = table
                    .sequence_values()
                    .flat_map(|v| v.map(lua_to_json))
                    .collect();
                Ok(JsonValue::Array(values?))
            }
            TableType::Map => {
                let map: Result<JsonMap<_, _>, _> = table
                    .pairs::<String, LuaValue>()
                    .flat_map(|p| p.map(|(k, v)| Ok::<_, LuaError>((k, lua_to_json(v)?))))
                    .collect();
                Ok(JsonValue::Object(map?))
            }
        },
        LuaValue::UserData(_) => Err(LuaError::runtime("unsupported userdata")),
        _ => Err(LuaError::runtime("unsupported type")),
    }
}
