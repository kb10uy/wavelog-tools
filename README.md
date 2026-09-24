# Wavelog Tools

Tools using Wavelog REST API v2.

## qcgen

Generates JSON data for QSL cards through a Lua script.

```sh
# From Wavelog (token is read from file or WAVELOG_TOKEN)
wavelog-tools qcgen assets/qcgen/qslcard-single.lua \
    --wavelog https://log.example.com \
    --wavelog-token-file ~/.config/wavelog-tools/token \
    --qso-since 2026-01-01 \
    -i instruments.toml -o operators.toml

# From local ADIF file
wavelog-tools qcgen assets/qcgen/qslcard-single.lua --adif qsos.adi
```

- Only QSOs with `QSL_SENT` of `R` or `Q` are processed unless `--all-qsos` is given
- Station information comes from `STATION_CALLSIGN` and `MY_*` fields
- `MY_STATE` is resolved to its name via Wavelog catalog API (disable with `--no-state-names`)
- Operator comes from `OPERATOR`; its display name can be looked up from operators file
- `QSL_VIA` is passed as `card.manager`
- `!inst:<key>` in `COMMENT` selects instrument; power is taken from `TX_PWR`, `--power`, then instrument default

See `assets/qcgen/` for example files and `assets/schope-types/` for Lua type definitions.
