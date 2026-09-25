# Wavelog Tools

Tools using Wavelog REST API v2.

## Configuration

Settings are read from `wavelog-tools/config.toml` in the user config directory
(`$XDG_CONFIG_HOME` or `~/.config` on Linux and macOS, `%APPDATA%` on Windows).
Use `-c/--config` to specify another file. See `assets/config.example.toml`.

- `[wavelog]`: URL and API token (`token`, `token_file`, or `WAVELOG_TOKEN` environment variable)
- `[operators.<CALLSIGN>]`: display name of operator

`instruments.toml` placed next to `config.toml` is also loaded by `qcgen` (see `assets/qcgen/instruments.example.toml`).

## qcgen

Generates JSON data for QSL cards through a Lua script.

```sh
# From Wavelog configured in config.toml
wavelog-tools qcgen assets/qcgen/qslcard-single.lua --qso-since 2026-01-01

# From local ADIF file
wavelog-tools qcgen assets/qcgen/qslcard-single.lua --adif qsos.adi
```

- Only QSOs with `QSL_SENT` of `R` or `Q` are processed unless `--all-qsos` is given
- Station information comes from `STATION_CALLSIGN` and `MY_*` fields
- `MY_STATE` is resolved to its name via Wavelog catalog API when Wavelog is configured (disable with `--no-state-names`)
- Operator comes from `OPERATOR`; its display name is looked up from `[operators]` in config
- `QSL_VIA` is passed as `card.manager`
- `!inst:<key>` in `COMMENT` (or `-I <key>`) selects instrument from `instruments.toml` and `-i` files; power is taken from `TX_PWR`, `--power`, then `default_power` of instrument

See `assets/qcgen/` for example files and `assets/schope-types/` for Lua type definitions.

## export

Exports QSOs from Wavelog as ADIF (ADI). Pages of `/api/v2/qso` are fetched and joined into one file.

```sh
# All QSOs to stdout
wavelog-tools export > all.adi

# 20m FT8 QSOs confirmed via LoTW or paper QSL in 2026
wavelog-tools export -o 2026-20m-ft8.adi --band 20m --mode FT8 --qsl-filter lotw,qsl --qso-since 2026-01-01 --qso-until 2026-12-31
```

Filters correspond to query parameters of `/api/v2/qso` and are also available in `qcgen` when reading from Wavelog.

| Option | Parameter | Description |
|---|---|---|
| `--station-id <ID,...>` | `station_id` | Station location IDs |
| `--callsign <CALLSIGN>` | `callsign` | Worked callsign (exact match) |
| `--band <BAND>` | `band` | Band such as `20m`; `SAT` for satellite QSOs |
| `--mode <MODE>` | `mode` | Mode or submode such as `SSB`, `FT8` |
| `--qsl-filter <TYPE,...>` | `qsl_filter` | Confirmed via any of `lotw`, `qsl`, `eqsl`, `qrz`, `clublog` |
| `--since-id <ID>` | `since_id` | QSOs with Wavelog ID greater than this |
| `--qso-since <DATE>` | `qso_since` | QSOs on or after the date |
| `--qso-until <DATE>` | `qso_until` | QSOs on or before the date |
