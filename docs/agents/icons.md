# Icon meanings

One glyph means one thing. A page's glyph appears for that page only (the
sidebar, the palette, links to it, its own empty states). Glyphs are the
constants of `crates/gui-kit/src/icons.rs`, bundled in
`crates/gui-kit/assets/icons/`. Page glyphs and hues are in
`app/gui/src/page_ui.rs`. Tests guarding these rules live in
`icons/tests.rs`, `page_ui/tests.rs`, `tokens/tests/tiles.rs` and
`widgets/tests/icon_tile.rs`.

## Rules

- Every glyph takes its colour from a theme token the caller passes as
  the tint. No glyph is drawn in a fixed colour.
- Info, warning, error and success use the bundled state glyphs (`state!`),
  never the desktop theme's, as themes draw them as coloured shapes. Each
  is the bare symbol with no square, triangle or circle around it.
- A state glyph shows as a status mark: the glyph in the status colour on
  a soft tile of it (`Tokens::status_tile`), at 3:1. The tile takes twice
  a banner's wash, so it shows on a banner. Banners, toasts, error rows,
  empty states, Home status rows and the Console's ended run all draw this
  mark (`Tone::status_mark`). Information takes the accent, a neutral
  banner the secondary text colour.
- Glyphs are symbolic. A glyph asks the desktop theme for its `-symbolic`
  name only (`Icon::theme_name`), else the bundled file is drawn.
- The only full colour icons are the two `coloured!` ones without a
  symbolic variant, listed with their reason in `FULL_COLOUR` of
  `icons/tests.rs`. They keep their colours on the coloured tile.
- A bundled glyph comes from breeze-dark, since Breeze's light files have
  dark ink that the tint turns near black. Its one colour is the ink
  `#fcfcfc`. A Breeze file with a second colour is recoloured or trimmed
  when bundled.
- Constants and catalog TOML names are the plain freedesktop names,
  without `-symbolic`.
- A theme glyph is drawn as a mask (`icons/mask.rs`): every colour turns
  white and the tint colours it. Breeze's symbolic shield would keep a
  green check otherwise. A theme file in `currentColor` is a glyph even
  for a coloured icon.
- The desktop's icon theme is read off the UI thread
  (`app/gui/src/desktop/icon_files.rs`) for every constant of
  `icons::ALL`. A theme with fewer symbolic glyphs mixes its own with the
  bundled ones, all in the same tint.
- A new meaning needs a glyph no other meaning uses. Add its row below.

## Meanings besides pages

| Meaning                    | Glyph                                         | Where                                                  |
| -------------------------- | --------------------------------------------- | ------------------------------------------------------ |
| Search                     | `SYSTEM_SEARCH`                               | sidebar Search (`page_ui::SEARCH_ICON`), search fields |
| Plain text file            | `TEXT_X_GENERIC`                              | Config Files sections without a known icon             |
| Information                | `HELP_ABOUT`                                  | banners, toasts, empty states                          |
| Warning                    | `DIALOG_WARNING`                              | banners, Home rows                                     |
| Error                      | `DIALOG_ERROR`                                | banners, toasts, empty states, Home rows               |
| Success                    | `DIALOG_POSITIVE`                             | success banners                                        |
| Read again, refresh        | `VIEW_REFRESH`                                | header refresh, status bar, Services Reload            |
| Restart needed             | `SYSTEM_REBOOT`                               | Home status row                                        |
| Reset to defaults          | `DOCUMENT_REVERT`                             | Maintenance Reset, Reset Snapper                       |
| Boot menu                  | `FORMAT_LIST_UNORDERED`                       | Boot Options menu heading                              |
| Health check               | `VIEW_TASK`                                   | Maintenance Repairs                                    |
| Web service, web page      | `GLOBE`                                       | SearXNG tile, wiki row actions, Config Files           |
| Exorcist                   | `DRAW_CROSS`                                  | About "Call the exorcist"                              |
| Game launchers             | `GAMES_CONFIG_BOARD`                          | Gaming catalog section                                 |
| Appearance                 | `PREFERENCES_DESKTOP_THEME_GLOBAL`            | Settings section, Boot Options "Look"                  |
| Behaviour, time            | `CHRONOMETER`                                 | Settings section, Home "last update" row               |
| Bluetooth                  | `PREFERENCES_SYSTEM_BLUETOOTH`                | Network radios, Components section                     |
| Keys, passwords, firewall  | `SECURITY_HIGH`                               | Maintenance keyring, Vaultwarden, Components Firewall  |
| Copy                       | `EDIT_COPY`                                   | row actions, property rows                             |
| Terminal, command          | `UTILITIES_TERMINAL`                          | row actions, kernel parameters, Home steps             |
| More actions               | `OVERFLOW_MENU`                               | row menus                                              |
| Repairs                    | `GAMES_SOLVE`                                 | Maintenance Repairs heading                            |
| Health fix                 | `GAMES_HINT`                                  | Maintenance "Repair system health"                     |
| Wiki, documentation        | `DOCUMENTATION`                               | About wiki rows                                        |
| Configuration file preview | `DOCUMENT_PREVIEW`                            | Config Files reading and no-files states               |
| Picked, done               | `CHECKMARK`                                   | Setup Assistant summary headings                       |
| Start, Stop                | `MEDIA_PLAYBACK_START`, `MEDIA_PLAYBACK_STOP` | Services row menu                                      |
| Restart                    | `MEDIA_PLAYLIST_REPEAT`                       | Services row menu                                      |
| Mask, Unmask               | `OBJECT_LOCKED`, `OBJECT_UNLOCKED`            | Services row menu                                      |

Page glyphs reused for their topic: `DRIVE_HARDDISK_ROOT` (Config Files
"Bootloader") and `OFFICE_REPORT` (Settings "Diagnostics").
