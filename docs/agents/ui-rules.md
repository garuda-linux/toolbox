# UI rules

Rules for pages, gui-kit widgets and the command line's output.

## Layout

- Toolbar spacing: 4 px between the controls of one group (a label and its
  dropdown), 8 px between groups (search, sort, tabs, refresh), as Kirigami's
  small and large spacing. The group gap is measured between what the groups
  paint, not their rects: a field's outline, a dropdown's pill, a button's
  glyph, and tabs as their pills inset `PILL_INSET` from their rects
  (`PageHeader::trailing_inset`, `header/tests/painted.rs`). Tabs and the
  refresh button stand 10 px apart because their rects touch, and 8 px would
  overlap them.
- Page header: a toolbar row as Kirigami's global toolbar, `toolbar_height`
  (46 px, `Sizes`) tall with the title at its left in the Kirigami level 1
  heading size (`PAGE_TITLE`, Regular weight, `Heading.qml`), and tabs and
  **Read again** at its right, centred on the row. A 1 px separator in
  `outline` always shows under the row. The purpose heads the body in
  secondary text, so it scrolls with the page. The controls go under the
  title only when the title cannot fit beside them. F5 presses **Read
  again** only while it is enabled and no dialog or focused terminal is in
  the way.
- A fixed toolbar under the header: search at the left, a labelled **Sort by** dropdown at the right. Filter chips
  scroll with the body.
  A chip group wraps onto as many lines as its width needs, with no
  sideways scroll, no edge fade and no scroll bar.
- The page paints under the header band. In opaque mode the band shows
  the live blur under `header_scrim`, a neutral layer thick enough for the
  header text and at least a third opaque, so an accent control scrolled
  under it blurs into a faint hue. Without the live blur it is solid
  `window`. With the glass `header_scrim` is transparent, so the band
  adds nothing over the live blur. With the glass and no live blur the
  band takes `window_glass`, so the page's words never show bare under
  the title (`widgets/page.rs`). The body starts `header_gap` below the
  separator.
- Switching page or tab starts at the top. Dialogs, the Console and a
  refresh keep the scroll position.
- Row actions sit at the right, on the middle of the whole row. On a row
  taller than about three lines they sit on its title line instead, so
  they stay beside the name they act on. When the text would get less than
  `MIN_TEXT_WIDTH` they drop under it (`ActionRow`, `split_row`).
- As in KDE's form delegates (Kirigami Addons `FormCard`), the leading icon
  or picture and the whole text block (title and detail) are centred on the
  row's height, and so are the actions (`ActionRow`). A row is as tall as
  its tallest part, with the icon's and the text's centres on one line.
- A card row's title line is at least `theme::PILL_HEIGHT` (32 px) tall
  with or without a control, and grows to its tallest control, so a line
  that holds a push button is 34 (`text_block_height_with`).
- Tag text sits on the capitals, as in the title line
  (`fonts::cap_centring`).
- Every single-line control is as tall as a push button, text + 18 = 34 px
  (`push_height`): line edits, search fields, secret and number fields, spin
  boxes, dropdowns, tabs and push buttons. A field and a button in one row
  share their edges. Multi-line text boxes grow with their text. Chips and
  tags are 27, Kirigami's chip at a 10 pt font (`CHIP_HEIGHT`,
  `TAG_HEIGHT`). Small buttons are 24 (`SMALL_BUTTON`). Item rows are
  text + 6, link rows too, and menu items are the text line plus 8, at
  least 30 (`menu::ITEM_HEIGHT`). Rows centre their controls, and the
  edges may differ.
- Tags have one size (`TAG_HEIGHT`) and sit on the line's middle.
- A row's title, its tags and every description line start at one x.
- Text is centred on its capitals, not on egui's line box (`fonts::cap_centring`). A `LayoutJob` in a title line takes
  `valign: Align::Center`.
- Every row menu entry has an icon, or none of the menu's entries do.
- A dropdown's open list is as wide as its button or its widest entry, at
  most `LIST_MAX_WIDTH`, and cuts a longer entry.
- A click anywhere on a row with a switch flips the switch. A click
  beside a row's button presses nothing, `Space` on the focused row does.
- One row grammar by meaning (`pages/toggles/control.rs`). A page of
  settings alone (System Tweaks, Services) has switches. A page that
  mixes settings and packages (Components, Gaming, Setup Assistant, App
  Catalog) has a button on every row: **Enable** or **Disable** for a
  setting or a service, **Install**, **Remove** or **Take back** and the
  word **Installed** for a package. Kernels has the same package
  buttons, as wide as theirs. Catalog and app rows end in
  the ⋯ menu before that control and cut their description after two
  lines.
- Every sidebar entry fits in its normal spacing down to 964 px window
  height and packed down to 902 px. Entries sit 36 px apart
  (`Sizes::nav_row_height`), as System Settings lays them out. Each
  highlight is 30 px tall, starts 6 px down its row and runs to the row's
  bottom, with a 9 px radius. Icon and label centre on the highlight.
  Below that the sidebar scrolls. A fade and a hairline mark a cut edge,
  the hairline alone with the glass. A group heading shows whole or not
  at all. The open page's entry never fades. A cut entry on the collapsed
  rail draws faded with no band over it, so the rail shows that more
  follows.

## Buttons and questions

- Button strengths: the doc comment of `crates/gui-kit/src/widgets/buttons.rs`.
- Changes that lose data or cut the session ask first, with a Danger
  button named after the change: destructive repairs, Remove orphans,
  Clean package cache, driver Unload, discarding configuration edits, and Stop,
  Restart and Mask on the display manager, NetworkManager, polkit, dbus
  and logind units.
- A dialog takes focus when it opens and gives it back to its opener.
- Danger is a plain push button, as KDE draws its destructive actions:
  no red fill, text in the normal text colour, and the question says what
  goes. Its icon is `edit-delete`, or the icon of the action when the
  question names one (a service's Restart, Stop or Mask).
- A question's buttons follow KDE's button layout (`QDialogButtonBox`
  KdeLayout): the answer left, Cancel at the far right, each with its
  standard icon (`dialog-ok`, `dialog-cancel`). Tab reaches the answer
  first (`widgets::confirm_buttons`).

## Look

- Every colour and size comes from the tokens in `crates/gui-kit`. Accent
  text is `accent_text` (4.5:1). Shadow blurs in these rules are Darkly's
  blur radii. egui's `Shadow::blur` spans both sides and is twice that
  (`buttons::look::BLUR_PER_RADIUS`).
  Selection in lists and palette rows fills the whole cell with
  `selection`, with no inset and no gap (`list/row.rs`,
  `press::item_look`). In opaque mode `selection` is `accent_tint`
  (Darkly's Highlight) and its text `on_tint`, the desktop's
  HighlightedText unless that reads below 4.5:1. Then the theme's text
  or window takes its place, whichever reads better, moved until it
  reaches 4.5:1 (`selection_text` in `tokens.rs`). A hover (`item_hover`) is
  `accent_tint` at 20 %, and a selected row under the pointer
  (`tint_hover`) is `accent_tint` lightened by 10 %. The sidebar keeps its
  inset highlight.
  Tabs (segmented controls) are 34 px high (`push_height`), joined with a
  1 px overlap (no gap), each tab 80 px wide at least. The text keeps 8 px
  of margin inside the chosen pill on each side, so 12 px from the tab's
  edge with the pill's 4 px inset (`TAB_MARGIN_X`). Every corner is
  rounded. An unchosen tab has 8 px corners. Nothing in the tab control
  is dark. The chosen tab is a 9 px pill in `chosen_tab` with a 1 px
  `card_outline`, inset 4 px at the sides and 2 px above and below in its
  tab rect, as tall as the face of a field or button beside it, and a
  shadow of blur 6, 1 px down, in `control_shadow`. In opaque mode
  `chosen_tab` is the `input` fill. An unchosen tab has no fill and its
  text stands alone, and under the pointer it takes `item_hover` in the
  same inset pill, never the whole tab rect. No base line runs under the
  tabs (`widgets/tests/selection.rs`).
- Keyboard focus is a fill or a line, never a ring, as Darkly draws it.
  Only a key shows it. A click focuses without showing it
  (`press::Pointer::of_keyboard`, `focus::focus_from_keyboard`). A focused
  push button, tool button, tab, chip or removable cross fills with
  `decoration_focus`. An unselected list row shows a 1 px `accent_tint`
  line along its bottom edge. Sidebar and menu items take their hover
  fill. The one ring is on the Console sheet's drag bar: a 2 px
  `accent_ring` inside its own edge (`focus::edge_ring`), meant for a box
  that fills its room to the edge, where a ring outside would leave the
  room.
- A focused text field shows a halo, as Darkly's focused line edit does:
  a ring of `accent_tint` at 60 % 1 px out from the fill and a shadow of
  blur 7, 1 px down, in `control_shadow` (`focus::field_halo`). It shows
  from a click too.
- Text reaches 4.5:1 on every fill it sits on. Push buttons and text
  fields draw a 1 px `card_outline` at rest, which reads at least 1.1:1
  against the window and the card (`tokens/tests/cards.rs`). Text fields
  draw Darkly's line edit: a fill (`tokens.input`, the View stepped toward
  the text where it is nearly the window, as a card is) inset 2 px, the
  `card_outline` at rest, no outline under the focus halo, and a shadow of
  blur 5, 1 px down, in black at 40 (`field::REST_BLUR`).
- A raised push button (Primary, Soft, Danger) keeps its `card_outline`
  at rest and under the pointer. The focus fill and the press fill
  replace it (`buttons.rs`). Its hover fill is `button_hover`, its pressed
  fill `button_pressed`. The shadow is 1 px down, in `control_shadow`,
  blurred 3 at rest, 6 on hover or focus and 2 while pressed
  (`buttons::look::push_blur`). A Ghost button has no outline and no
  shadow.
- Check boxes are 16 px boxes with 5 px corners and no frame. On, they
  fill with `accent_tint` and the tick is `on_tint`.
- A field's glyph, unit and buttons are atoms of its `TextEdit`
  (`widgets/field.rs`), so the whole pill focuses the text.
- A number field with a unit centres both as one group ("5 s"). A number
  alone ends at the right.
- Push buttons, dropdowns and tool buttons have `theme::CONTROL_RADIUS`
  corners (7 px, tool buttons 9 px). Text fields, spin boxes and the search
  field have `theme::FIELD_RADIUS` corners (8 px), as Darkly's line edits.
  Filter chips are Kirigami chips with 5 px corners, so what narrows a
  list reads apart from a control.
- A disabled control has one look: egui's fade of a disabled Ui (`disabled_alpha`) over its own colours, never a swapped
  colour or a
  second fade (`widgets/press.rs`). A Ghost button stays transparent.
- Text sizes come from the steps in `fonts.rs` (`TITLE`, `PAGE_TITLE`,
  `SECTION_TITLE`, `SECTION`, `BODY`, `SMALL`), never a multiple of one.
- Text weights: Regular, Medium and SemiBold all draw in the desktop's own
  weight (`kdeglobals` `[General] font=`, its third field), Bold is the
  desktop's bold face. KDE's Inter Bold 10 is therefore bold everywhere.
  GTK reads its font and weight from its settings, and a desktop without any
  gets the bundled Inter at 400 and 700.
- A Soft button on a dialog takes the plain push button fill. Dialogs dim the
  window under them with `scrim`.
- Frosted surfaces: see [Frost](architecture.md#frost).
- Menus, popups and the header keep their frost and blur. They have a 9 px
  radius, a 1 px outline (white at 30 % in dark themes) and two shadow
  layers: 5 px down with blur 10 at 44 %, and 3 px up with blur 5 at 24 %
  (`toasts::popup_shadows`, egui's 20 and 10).
  Menu items fill with the hover colour under the pointer and with
  `decoration_focus` when pressed. A separator is 1 px, inset 10 px.
- Scrollbars are painted by the app, as Darkly's are. The strip is 21 px,
  the handle 4 px wide and 8 px on hover, centred, with 6 px end insets
  and at least 20 px long. The handle is WindowText at 35 % at rest and
  the hover colour under the pointer, and `decoration_focus` while the
  parent has the focus. On hover a groove of 8 px in WindowText at 30 %
  shows behind it. A scrollbar takes room from the page.
- Tooltips use the Tooltip colour, a 9 px radius, no outline and 3 px of
  padding. A progress track is WindowText at 30 %, 4 px high with 2 px
  corners. Its fill is `accent` for an accent share and the status colour
  for the other tones (`widgets/meter.rs`).
- Cards are Darkly group boxes in both themes. In opaque mode the fill
  `card_glass` is `card` = `mix(Window, View, 0.3)`, stepped 4 % toward
  the text where View is nearly the Window colour (contrast under 1.1),
  with a 1 px `card_edge` (`card_outline`, the Window mixed 25 % toward
  WindowText) and a shadow of blur 4, 1 px down, in `card_shadow`
  (`control_shadow`). The glass rule below lists the glass values. A card
  fills the page body's width less its margins, with `CARD_RADIUS` 8 px
  corners (`widgets::card`).
- Spacing follows Kirigami's units (`theme::GRID_UNIT` 18, `SMALL_SPACING`
  4, `LARGE_SPACING` 8), see `tokens::Sizes`. A card row pads by large plus
  small spacing on both axes, a card header by large plus small above and
  small below, and a page keeps one grid unit around its content. They
  draw no light along their top (`widgets::light_rim`).
  Metric and stat tiles and link tiles keep their rim. The raised menus,
  popovers, notes and dialogs keep the 1 px `top_light` line along their
  top edge in dark themes (`widgets::paint_top_light`).
- The glass follows Zed's Catppuccin Blur theme. The window's background
  (`tokens.window_glass`) is the one translucent layer, 84 % opaque
  (`WINDOW_OPACITY`, Zed's `background`), while the desktop blur is on and
  offered (`theme::set_translucent`), else opaque. Nothing stacks on it: the
  sidebar, the page area, the header band, the changes footer, the status
  bar, the Console sheet and `tokens.band` draw no fill, and `sidebar`,
  `pane` and `header_scrim` are transparent too. Cards (`card_glass`) take
  no fill and no shadow, and only their edge (`card_edge`) and the lines
  between their rows (`row_divider`) show, at the button colour at 8 %
  (`separator`, Zed's `border`). A cut sidebar entry has no fade with the
  glass (`sidebar_fade`), as an opaque fade would show as a strip. With
  the glass the chosen tab is the crust at 69 % (`chosen_tab`, Zed's
  `tab.active_background`), hovers the crust at 56 % (`item_hover`, Zed's
  `ghost_element.hover`). Selections with the glass are the crust moved
  31 % toward the accent, solid (`selection`, text `selection_text` at
  4.5:1 on it), and a selected row under the pointer keeps that fill
  (`tint_hover`), as a stronger step falls under 4.5:1 in the light theme
  (`tokens/glass.rs`). Menus, dropdown lists and popups are opaque
  (`frost_raised`), and a dropdown entry under the pointer is
  `popup_hover`, a solid step over the popup. Controls, banners, tiles,
  metrics keep their solid fills. `window` itself stays opaque, since
  widgets mix it into frames and footers. In opaque mode `sidebar` is
  `window` under `header_scrim`, so sidebar and header band read as one
  surface.
- Sidebar highlights sit 6 px in from the sidebar's edges
  (`press::paint_item`). The sidebar's own lines and the entries'
  icons and text start `NAV_INSET` from its edge.
- Section headings are FormCard's `FormHeader`: Bold at the body size
  (`BODY`), with no tile and no glyph (`widgets/heading.rs`).
  `SECTION_TITLE` (the Kirigami level 2 size) is only for an empty
  state's title (`widgets/empty.rs`).
- Icons follow [icons.md](icons.md). Row and link icons are plain colour icons
  at Kirigami sizes, with no tile behind them.
  An empty state is a Kirigami placeholder: one 96 px icon in the
  disabled text colour (`tokens.dim`), its title in Bold at the level 2
  size in the secondary colour (4.5:1 on every fill), the
  explanation below in the secondary colour.
  Rows with a status keep the status wash.
- An app without a picture, or while it loads, shows a letter tile (`widgets::paint_letter_tile`). Its hue comes from
  the package name, so
  it is the same on every page and in every language. It is decorative.
- Toggle rows always keep the picture slot, so every row aligns.
- A page body fills the window's width less its margins, headings, cards and
  notes alike (`BodyShape::Column`, the default, and `Wide` on list pages).
  There is no narrower column for setting pages. The page toolbar's
  controls end where the cards end, beside the scroll strip, and its band
  spans the window. The User guide, App Catalog and Packages use
  `BodyShape::Panes`, so their panes scroll on their own.
- A page the pointer opened fades in and rises (`motion::PAGE_ENTER`).
  Only the body moves. Another tab, reduced motion and keyboard
  navigation show in place.

## Lists and states

- Filters are chips with an **All** chip. Counts follow a search. Chips
  of a toggle page count as "2/8" (on of total), as section headings do,
  and as "?/8" while a state they count is unread.
- A picked chip that hides every match of a search keeps its pick. The
  empty state names it and offers **Show all N matches**.
- A state (Running, Installed, Failed) is a `widgets::status_dot`, a dot
  and the word in secondary text, as a pill would repeat down the page.
- A queued change is an accent status dot everywhere (`widgets::queued_tag`), never a pill.
- A count is no state and shows as plain secondary text.
- A state that changes while it shows is a `widgets::status_word`, so
  screen readers hear it change.
- In a long live list (Services) only the focused row and every failure
  are a `status_word`, the other rows a `status_dot`, so a relist does
  not read the whole list.
- A part that could not be read gets one banner with **Read again**, not a
  line on every row.
- A page that could not be read at all shows a centred empty state with **Read again** under it
  (`widgets::empty_state_action`). Its button
  sends what the header's **Read again** and F5 send.
- A page that waits for its main content shows skeleton rows in the card
  shape of the rows to come (`widgets::skeleton_list`), named by what is
  read. A lighter `shimmer` band sweeps them in one phase for the whole
  list (`motion::SHIMMER_PERIOD`). Out of sight or with less motion asked
  for, the bars stand still and ask for no frames. Three dots are only
  for a short wait inside a loaded page.
- Cut text ends in "…" and shows whole in a tooltip and to screen readers.
- Copy buttons only on technical values (versions, addresses, device
  paths, UUIDs).
- A guide code block's copy button is the small button over its top right
  corner (`widgets::corner_button`). It fades in while the pointer is on
  the block or the button holds the focus, and stays in the Tab order
  and the accessibility tree while hidden. Code runs under it.

## Shell

- The queue is a KDE footer toolbar under the page (`shell/changes/bar.rs`):
  the page area's full width, 47 px tall (KCM footer margins of 13 px and a
  34 px push button), with a Kirigami separator line above it. "N changes
  queued" sits at the left, **Review** (`document-preview`) and **Apply**
  (`dialog-ok-apply`, the default button) at the right. Review opens its
  popover upward, which keeps the frost. A page with its own way to apply
  hides the footer (`PageView::queue_shown`).
- The Console is a resizable sheet over the page. The page is not dimmed
  and stays usable. Its scroll area ends at the sheet's top, so a control
  brought into view lands above the sheet. A click on the sheet never
  reaches the page.
- The Console terminal owns every key except Ctrl+Tab and Ctrl+Shift+Tab,
  which move focus out.
- Toasts are Kirigami passive notifications: at the bottom centre of the
  page area, 8 px above its edge, the newest on top, `tight_gap` apart.
  They take the Complementary colour set at 90 % opacity, no icon and no
  Close button, and a click closes one. A note stays seven seconds, the
  same for an error (`toasts::TIME`), and grows with its text up to
  `toasts::MAX_WIDTH` (450 px) before it wraps.
- A banner is a Kirigami `InlineMessage`: a 1 px frame in its type
  colour, the card with that colour at 20 % over it (`banner::WASH`,
  Kirigami's `opacity: 0.20`), its status mark (`paint_status_tile`) and
  its actions as push buttons at the right. Information takes the view's
  active text colour (`tokens.active`), and a neutral note too.
- A quieter wash, the card with the status colour at 12 %
  (`tokens.banner_wash`, `BANNER_WASH`), fills lines that are no banner:
  added and removed lines in the configuration review and the merge
  details, and the guide's warning callouts. A status glyph's tile takes
  24 % (`STATUS_TILE_WASH`).
- A dialog is a Kirigami `Dialog`: `CARD_RADIUS` corners (8 px), a 1 px
  frame between the card and the text colour, Darkly's two popup shadow
  layers as menus have them (`toasts::paint_popup_shadow`), and a rule
  under the title.
- Tab order: sidebar, header, page body, status bar last. Controls inside
  a row in visual order.
- A list and a chip group take one Tab stop (`widgets/roving.rs`). A list's
  is its last focused row, a chip group's the selected chip. Up and Down (Left and Right on chips) move it, Home and End
  go to the ends. Tests
  reach a row by these keys, not by an accessibility focus, which egui
  ignores on a widget that is not a Tab stop.

## Texts

- en-US follows the KDE HIG capitalization. Title Case for buttons, menu
  items, tabs, page and section headings, window and dialog titles and
  toolbar actions. Sentence case for labels, descriptions, check box and
  switch labels, tooltips, messages and notes. Product and technical names
  keep their own case. The German texts are not changed by this rule.
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
- Lists and `show` page through a pager on a terminal (`crates/commands/src/pager.rs`), never under `--json`, questions
  or
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
