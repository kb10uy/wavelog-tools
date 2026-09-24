---@param text string|nil
---@return string|nil
local function capitalize_if_upper(text)
    if text == nil or text == "" then
        return nil
    end
    if text:upper() ~= text then
        return text
    end
    return (text:lower():gsub("(%a)([%w']*)", function(head, tail)
        return head:upper() .. tail
    end))
end

---@param station QslStation
---@return string
local function format_location(station)
    local candidates = {
        capitalize_if_upper(station.city),
        station.state_name or station.state,
        capitalize_if_upper(station.country),
    }
    local parts = {}
    for i = 1, 3 do
        if candidates[i] ~= nil and candidates[i] ~= "" then
            table.insert(parts, candidates[i])
        end
    end
    return table.concat(parts, ", ")
end

---@param args table<string,string>
---@param entries QslCardEntry[]
local function generate(args, entries)
    local converted_entries = {}
    for i, e in ipairs(entries) do
        local bureau_call = e.qso.call
        local call = e.qso.call
        local call_suffix_index = call:find("/")
        local via = false
        local datetime = e.qso.datetime
        local timezone = "UTC"

        if e.info.card.manager then
            bureau_call = e.info.card.manager
            via = true
        elseif call_suffix_index ~= nil then
            bureau_call = bureau_call:sub(1, call_suffix_index - 1)
        end

        if e.qso.mode ~= "FT8" then
            datetime = datetime:to_offset("+09:00")
            timezone = "JST"
        end

        table.insert(converted_entries, {
            via = via,
            bureau_call = bureau_call,
            call = call,
            date = datetime.date_str,
            time = datetime.time_str,
            timezone = timezone,
            report = e.exchange.tx_report or "",
            freq = e.qso.freq_str,
            mode = e.qso.mode,
            rig = e.info.instrument.rig or "",
            power = e.info.instrument.power and tostring(e.info.instrument.power) or "",
            antenna = e.info.instrument.antenna or "",
            location = format_location(e.info.station),
            operator = e.info.operator.name or e.info.operator.callsign or "",
            received = e.info.card.received,
        })
    end

    return converted_entries
end

return {
    generate = generate,
}
