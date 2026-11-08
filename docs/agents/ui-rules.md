# UI rules

Rules for pages, gui-kit widgets and the command line's output.

## Layout

- Page header: title, purpose, then tabs and **Read again** on the
  title's line, the same on every tab. F5 presses **Read again** only
  while it is enabled and no dialog or focused terminal is in the way.
- A fixed toolbar under the header: search at the left, a labelled
  **Sort by** dropdown at the right. Filter chips scroll with the body.
- The page paints under the header band. The band shows the live blur
  under `header_scrim`, a neutral layer just thick enough for the header
  text. Without the live blur it is solid `window`. A hairline shows once
  content runs under it. The body starts `header_gap` below it.
- Switching page or tab starts at the top. Dialogs, the Console and a
  refresh keep the scroll position.
- Row actions sit at the right, on the middle of the whole row. When the
  text would get less than `MIN_TEXT_WIDTH` they drop under it
  (`ActionRow`, `split_row`).
- The leading tile or picture and the title start at the row's top. A
  text shorter than the tile is not centred on it.
- Controls keep `theme::PILL_HEIGHT` inside a card row. A card row's title
  line is that tall with or without a control, so every row of a card
  keeps one height.
- From a card row's top edge to the title's capitals is the same room as
  from the last baseline to its bottom edge (`title_line`, `ActionRow`).
  Tiles, pictures and tags start at the capitals. A text shorter than a
  control is centred on the row.
- Tags have one size (`TAG_HEIGHT`) and sit on the line's middle.
- A row's title, its tags and every description line start at one x.
- Text is centred on its capitals, not on egui's line box
  (`fonts::cap_centring`). A `LayoutJob` in a title line takes
  `valign: Align::Center`.
- Every row menu entry has an icon, or none of the menu's entries do.
- A dropdown's open list is as wide as its button or its widest entry, at
  most `LIST_MAX_WIDTH`, and cuts a longer entry.
- A click anywhere on a row with a switch flips the switch.
- Every sidebar entry fits down to 796 px window height, below that the
  sidebar scrolls. A fade and a hairline mark a cut edge. A group heading
  shows whole or not at all. The open page's entry never fades. A cut
  entry on the collapsed rail draws faded with no band over it, so the
  rail shows that more follows.

## Buttons and questions

- Button strengths: the doc comment of `crates/gui-kit/src/widgets/buttons.rs`.
- Changes that lose data or cut the session ask first, with a Danger
  button named after the change: destructive repairs, Remove orphans,
  Clean package cache, driver Unload, discarding configuration edits, and Stop,
  Restart and Mask on the display manager, NetworkManager, polkit, dbus
  and logind units.
- A dialog takes focus when it opens and gives it back to its opener.

## Look

- Every colour and size comes from the tokens in `crates/gui-kit`. Focus
  rings use `accent_ring` (3:1), accent text `accent_text` (4.5:1).
  Selection uses the accent tint.
- Text reaches 4.5:1 on every fill it sits on. Fields and controls have a
  3:1 edge, on filled fields a rim round the whole shape (`field_rim`). A
  focused field shows the focus ring in place of the rim, never both.
- A number field with a unit centres both as one group ("5 s"). A number
  alone ends at the right.
- Frosted surfaces: see [Frost](architecture.md#frost).
- The window is opaque. `sidebar` is `window` under `header_scrim`, so
  sidebar and header band read as one surface.
- Icons follow [icons.md](icons.md). Row, link and section icons sit in a
  `widgets::icon_tile` in the open page's group hue, set by the shell
  (`tokens::set_page_hue`). Pages never pick one. Rows with a status keep
  the status wash.
- An app without a picture, or while it loads, shows a letter tile
  (`widgets::paint_letter_tile`). Its hue comes from the package name, so
  it is the same on every page and in every language. It is decorative.
- Toggle rows always keep the picture slot, so every row aligns.
- Page body at most `theme::CONTENT_WIDTH`, centred. Only the User guide
  uses `BodyShape::Panes`, so its panes scroll on their own.
- A page the pointer opened fades in and rises (`motion::PAGE_ENTER`).
  Only the body moves. Another tab, reduced motion and keyboard
  navigation show in place.

## Lists and states

- Filters are chips with an **All** chip. Counts follow a search. Chips
  of a toggle page count as "2/8" (on of total), as section headings do.
- A state (Running, Installed, Failed) is a `widgets::status_dot`, a dot
  and the word in secondary text, as a pill would repeat down the page.
- A queued change is an accent status dot everywhere
  (`widgets::queued_tag`), never a pill.
- A count is no state and shows as plain secondary text.
- A state that changes while it shows is a `widgets::status_word`, so
  screen readers hear it change.
- A part that could not be read gets one banner with **Read again**, not a
  line on every row.
- Cut text ends in "…" and shows whole in a tooltip and to screen readers.
- Copy buttons only on technical values (versions, addresses, device
  paths, UUIDs).

## Shell

- The Changes pill sits in its own band at the bottom right. A page with
  its own way to apply hides it (`PageView::queue_shown`).
- The Console is a resizable sheet over the page. The page is not dimmed
  and stays usable. Its scroll area ends at the sheet's top, so a control
  brought into view lands above the sheet. A click on the sheet never
  reaches the page.
- The Console terminal owns every key except Ctrl+Tab and Ctrl+Shift+Tab,
  which move focus out.
- Toasts start at the top right of the page area and grow with their text
  up to `toasts::MAX_WIDTH` before it wraps.
- Tab order: sidebar, header, page body, status bar last. Controls inside
  a row in visual order.

## Texts

- Sentence case, except page names and product names.
- Transifex: [translations.md](translations.md).

## Command line

- Output goes through `toolbox_cli::output::Paint`: a bold title with a
  dimmed summary, rows indented by two, a state mark per row (● on, ○ off,
  ? unknown, ✗ failed), later columns dimmed, `error:`, `warning:` and
  `hint:` lines. Marked lists have no header row. Colour follows
  `--color` (`output::ColorEnv`).
- `-q` leaves out spinners and `hint:` lines, never an error.
- Long cells are cut to the terminal width, never the name, never in a
  pipe.
- A program's failure is one `error:` line. Its own words show only with
  `-v`, else a hint names the flag. A text with a second line is an error
  and its hint.
- Text errors go to standard error. Under a read's `--json` standard
  output holds the JSON alone, standard error one `{"schema":1,"error":…}`
  object per error once the exit code is known (`out::kept_errors`).
  A change under `--json` keeps its JSON Lines errors on standard output.
- Output is English and JSON is locale free. JSON goes through
  `output::json` or `json_line`. Give a plain value a named field. A row
  is known by `id`. `name` is a display name only. `state` is a row's
  condition in one word, other states get their own key.
- `list` for collections, `show` for one state view. Every read takes
  `--json`, every `list` takes `--search`.
- Lists and `show` page through a pager on a terminal
  (`crates/commands/src/pager.rs`), never under `--json`, questions or
  running output.
- A change prints the plan and asks `Continue? [y/N]`. No terminal and no
  `--yes` refuses. `--dry-run` prints the plan and exits 0. Steps are
  counted (`[2/5]`) and timed.
- Ctrl+C, SIGTERM and SIGHUP stop a change alike (130, 143, 129) and end
  `status --watch` with 0.
- `status` and `doctor` exit 9 when something needs attention.
- Exit codes go in the man page and `docs/src/cli.md`, not in `--help`.
- Completion is dynamic (`toolbox_cli::complete`) and never needs root or
  the helper.
- Logging: see [logging.md](logging.md).
