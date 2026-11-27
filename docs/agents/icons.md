# Icon meanings

One icon means one thing. A page's icon appears for that page only (the
sidebar, the palette, links to it, its own empty states). Glyphs are the
constants of `crates/gui-kit/src/icons.rs`, colour icons those of
`crates/gui-kit/src/icons/colour.rs`. The list of every icon is
`icons/all.rs`. Bundled files live in `crates/gui-kit/assets/icons/`
(glyphs) and `assets/icons/colour/` (colour icons). Page icons are in
`app/gui/src/page_ui.rs`. Tests guarding these rules live in
`icons/tests.rs`, `page_ui/tests.rs`, `tokens/tests/tiles.rs` and
`widgets/tests/icon_tile.rs`.

## Rules

- Two kinds of icon. A glyph is a monochrome symbol the text colour tints
  (buttons, fields, menus, row marks it has no colour for). A colour icon
  is Breeze's own colour file, drawn in its colours and never tinted (pages,
  status, rows). KDE draws its icons the same way.
- Both ask the desktop theme for their file by name: a glyph its
  `-symbolic` name, a colour icon the plain name (`Icon::theme_name`).
  Each then falls back to its bundled file, so one name can be a glyph and
  a colour icon at once without a collision.
- Monochrome action icons (`GO_HOME`, `GO_PREVIOUS`, `EDIT_FIND`,
  `CONFIGURE`, `HELP_CONTENTS`, `HELP_ABOUT`) ask for the plain name and
  take the tint.
  Breeze draws these in `currentColor`, so no colour file exists for them.
- Status icons are colour icons: `DIALOG_INFORMATION`, `DIALOG_WARNING`,
  `DIALOG_ERROR`, `DIALOG_POSITIVE`, drawn plain at 22 px in lines
  (`widgets::status_tile`, `Tone::status_mark`). Information and a neutral
  note take the information icon, as KDE has no neutral one. Banners,
  toasts, error rows, empty states and Home status rows show this mark.
- Row icons: `Icon::for_row` maps a glyph to the colour icon KDE apps use
  for the same thing (`colour::of_row`). A glyph with no colour file keeps
  its symbol. Row icons draw on no tile, at 22 px (`ICON_EDGE`), in the
  text colour for a glyph.
- Sidebar entries draw their page icon at 22 px in a 36 px entry
  (`widgets::nav`, `Sizes::nav_row_height`). A monochrome page icon takes
  the sidebar text colour.
- Section headings draw no icon: Kirigami's headings are text
  (`SectionHeading`, `section_heading`, `section_intro`). A section's icon
  stays only where its rows draw one.
- A bundled glyph comes from breeze-dark, since Breeze's light files have
  dark ink that the tint turns near black. Its one colour is the ink
  `#fcfcfc`. A Breeze file with a second colour is recoloured or trimmed
  when bundled.
- A bundled colour icon comes from breeze-dark too (the first of the
  Breeze folders `apps`, `categories`, `preferences`, `devices`, `places`,
  `mimetypes`, `status` and `actions` with the file). Its `ColorScheme-Text`
  parts keep `currentColor` and take the ink of the place the icon is drawn
  in (`icons/ink.rs`), as KDE recolours that class. The theme's own files do
  the same. Breeze is LGPL, see `assets/icons/COPYING-ICONS`.
- Constants and catalog TOML names are the plain freedesktop names,
  without `-symbolic`. `icons::named` prefers a glyph of a shared name.
- A glyph's theme file is a mask (`icons/mask.rs`): every colour turns
  white and the tint colours it. Breeze's symbolic shield would keep a
  green check otherwise. A colour icon's theme file keeps its colours, and
  its `currentColor` parts take the ink.
- The desktop's icon theme is read off the UI thread
  (`app/gui/src/desktop/icon_files.rs`) for every constant of `icons::ALL`. A
  theme with fewer files mixes its own with the bundled ones.
- A new meaning needs an icon no other meaning uses. Add its row below.
- A toggle entry's `glyph` (a bundled name) replaces its picture in the
  picture slot as a plain row icon. Every Components entry has one: its
  section's topic glyph, `SYSTEM_RUN` for a service, `USER_OTHERS` for a
  group.
- Page icons (KDE System Settings names): Home
  `GO_HOME`, Search `EDIT_FIND`, Setup Assistant `SYSTEM_SOFTWARE_INSTALL`,
  Components `PREFERENCES_DESKTOP_PERIPHERALS`, Gaming
  `PREFERENCES_DESKTOP_GAMING`, App Catalog `PLASMADISCOVER`, Packages
  `PACKAGE_X_GENERIC`, Services `PREFERENCES_SYSTEM_SERVICES`, System Tweaks
  `PREFERENCES_SYSTEM`, Kernels `PREFERENCES_SYSTEM_LINUX`, Languages
  `PREFERENCES_DESKTOP_LOCALE`, Users and Groups
  `PREFERENCES_SYSTEM_USERS`, Config Files `PREFERENCES_OTHER`, Network
  `PREFERENCES_SYSTEM_NETWORK`, Boot Options `PREFERENCES_SYSTEM_STARTUP`,
  Maintenance `APPLICATIONS_UTILITIES`, Diagnostics `HWINFO`, Settings
  `CONFIGURE`, User guide `HELP_CONTENTS`, About `HELP_ABOUT`.
  App Catalog uses Breeze's `plasmadiscover` only: a theme without it
  falls back to the bundled `plasmadiscover` file, not `applications-all`.

## Meanings besides pages

| Meaning                     | Glyph                                                          | Where                                                                     |
| --------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Search                      | `SYSTEM_SEARCH`                                                | search fields, no-match empty states, the palette's search commands       |
| Configuration file to merge | `VCS_DIFF`                                                     | Home status row, the files of the merge dialog                            |
| Plain text file             | `TEXT_X_GENERIC`                                               | Config Files sections without a known icon                                |
| Information                 | `DIALOG_INFORMATION`                                           | banners, toasts, empty states, Console notes                              |
| Warning                     | `DIALOG_WARNING`                                               | banners, Home rows                                                        |
| Error                       | `DIALOG_ERROR`                                                 | banners, toasts, empty states, Home rows                                  |
| Success                     | `DIALOG_POSITIVE`                                              | success banners                                                           |
| Read again, refresh         | `VIEW_REFRESH`                                                 | header refresh, status bar, Services Reload                               |
| Restart needed              | `SYSTEM_REBOOT`                                                | Home status row                                                           |
| Reset to defaults           | `DOCUMENT_REVERT`                                              | Maintenance Reset, Reset Snapper                                          |
| Health check                | `VIEW_TASK`                                                    | Maintenance Repairs                                                       |
| Web service, web page       | `GLOBE`                                                        | SearXNG tile, wiki row actions, Config Files                              |
| Exorcist                    | `DRAW_CROSS`                                                   | About "Call the exorcist"                                                 |
| Game launchers              | `GAMES_CONFIG_BOARD`                                           | Gaming catalog section                                                    |
| Behaviour, time             | `CHRONOMETER`                                                  | Settings section, Home "last update" row                                  |
| Bluetooth                   | `PREFERENCES_SYSTEM_BLUETOOTH`                                 | Network radios, Components section                                        |
| Wired network card          | `NETWORK_WIRED`                                                | Network page interface cards of a wire, drivers and hardware empty states |
| Keys, passwords, firewall   | `SECURITY_HIGH`                                                | Maintenance keyring, Vaultwarden, Components Firewall                     |
| Copy                        | `EDIT_COPY`                                                    | row actions, property rows                                                |
| Terminal, command           | `UTILITIES_TERMINAL`                                           | row actions, kernel parameters, Home steps                                |
| More actions                | `OVERFLOW_MENU`                                                | row menus                                                                 |
| Repair rows                 | `CONFIGURE`, `SECURITY_HIGH`, `RUN_BUILD`, `AUDIO_VOLUME_HIGH` | Maintenance Repairs, after the health rows (`repair_icon`)                |
| Health fix                  | `GAMES_HINT`                                                   | Maintenance "Repair system health"                                        |
| Wiki, documentation         | `DOCUMENTATION`                                                | About wiki rows                                                           |
| Configuration file preview  | `DOCUMENT_PREVIEW`                                             | Config Files reading and no-files states                                  |
| Picked, done                | `CHECKMARK`                                                    | Setup Assistant summary headings                                          |
| Start, Stop                 | `MEDIA_PLAYBACK_START`, `MEDIA_PLAYBACK_STOP`                  | Services row menu                                                         |
| Restart                     | `MEDIA_PLAYLIST_REPEAT`                                        | Services row menu                                                         |
| Mask, Unmask                | `OBJECT_LOCKED`, `OBJECT_UNLOCKED`                             | Services row menu                                                         |
| Show, hide a password       | `VIEW_VISIBLE`, `VIEW_HIDDEN`                                  | password fields (`widgets::secret_field`)                                 |
| Login shell                 | `DIALOG_SCRIPTS`                                               | Tweaks "Login shell" row                                                  |
| DNS servers                 | `NETWORK_SERVER`                                               | Tweaks "DNS server" row                                                   |
| Wi-Fi backend               | `NETWORK_WIRELESS_SIGNAL_GOOD`                                 | Tweaks "Wi-Fi backend" row                                                |
| Ad blocking                 | `DIALOG_CANCEL`                                                | Tweaks "Ad blocking" row                                                  |
| Mirrorlist refresh          | `VIEW_LIST_DETAILS`                                            | Tweaks Garuda Update "Skip mirrorlist refresh" row                        |
| AUR updates                 | `APPLICATIONS_DEVELOPMENT`                                     | Tweaks Garuda Update "Update AUR packages" row                            |
| Non-interactive pacman      | `DIALOG_OK`                                                    | Tweaks Garuda Update "Non-interactive pacman" row                         |
| Disk space check            | `DRIVE_HARDDISK`                                               | Tweaks Garuda Update "Skip disk space check" row                          |
| Installed kernels           | `CHECKBOX`                                                     | Kernels "Installed" rows                                                  |
| Zoom in, out, reset         | `ZOOM_IN`, `ZOOM_OUT`, `ZOOM_ORIGINAL`                         | palette commands                                                          |
| Kernels to install          | `FOLDER_DOWNLOAD`                                              | Kernels "Available" rows                                                  |
| Queued changes: Apply       | `DIALOG_OK_APPLY`                                              | Changes footer and popover Apply (`shell/changes/buttons.rs`)             |
| Queued changes: Review      | `DOCUMENT_PREVIEW`                                             | Changes footer and popover toggle (Review, Hide)                          |

Page glyphs reused for their topic: `DRIVE_HARDDISK_ROOT` (Config Files
"Bootloader") and `OFFICE_REPORT` (Settings "Diagnostics").
