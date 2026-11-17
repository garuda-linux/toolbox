# User documentation

The user guide is an mdBook in `docs/src/`, built by
`scripts/build-docs.sh` (also the Cloudflare Pages build). Page titles and
their `docs/src/SUMMARY.md` entries stay in sync.

## Screenshots

- `docs/src/images/` holds the guide's 24 pictures, rendered by the kittest
  test `guide_images::every_guide_picture_matches_its_render`
  (`app/gui/tests/integration/guide_images.rs`) from the page fixtures, each
  in the dark and the light theme. `<name>.png` is the dark capture,
  `<name>-light.png` the light one.
- The guide shows `<name>.png` in the dark theme and `<name>-light.png`
  beside it in the light theme (`toolbox_guide::image_for`). A picture
  without a light file shows its dark one in both themes.
- A whole page crops to the page pane (accessible name `Page`). A dialog,
  a popover, the command palette and the Console crop to their own box.
  Every crop keeps a 24 px margin, so the shadow and outline stay whole.
- Regenerate after a change to a page or its fixture, with the
  `UPDATE_GUIDE_IMAGES=1` variable set for the test (`nix develop`, then
  `cargo nextest run -p toolbox-gui --features testing -E 'test(every_guide_picture)'`).
- Without the variable the same test fails when a committed picture
  differs from a fresh render. Regenerate again when `guide.png` or
  `guide-light.png` changes: the guide page shows the other pictures, which
  the binary embeds at build time.
- A new picture needs an entry in `ALL` in
  `app/gui/tests/integration/guide_images/captures.rs`, a function that draws
  its state in a theme, and the image in the page's Markdown.
- Fixtures that draw a guide picture take a `Theme` and the desktop look of
  that theme (`look_for` in `tests/integration/look.rs`). A `*_on(theme)`
  function sits beside each fixture, and the old name keeps the dark theme.
- Shots of "not available" states belong only in `troubleshooting.md`.

## Rules

- A page covers every button, menu entry, setting, dialog and message of
  its area in `app/gui/src/pages/**`. Every behaviour claim is backed by
  the code. Every bold UI label matches an `en-US` Fluent text exactly.
- Grep `docs/src` for `page.md#anchor` before renaming a heading.
- Wrap at 80 columns, `-` bullets, `1.` lists, no semicolons outside code.
- Plain second person, goal first ("To add a package, search for it and
  click **Add**."), never blaming the user. It is a desktop app: no
  "refresh the page".
- Headings name what the user wants to do. After each task, say what the
  user sees when it worked.
- Root, system changes and data loss get `> **Warning:**` or
  `> **Note:**` with how to undo.
- Real package, service and kernel names as examples. The app's own
  terms, one word per thing. A short why for non-obvious limits.
- Say where installed system and live ISO, Garuda and other Arch systems,
  or Plasma and other desktops differ.
- Descriptive alt text, buttons named by label or tooltip, keyboard
  shortcuts listed.
- Link the matching `troubleshooting.md` entry where a task can fail.
- Help channel: `https://gitlab.com/garuda-linux/applications/toolbox/-/issues`.

## Shapes

- Task page: a one or two sentence intro, `## Before you start` only for
  real prerequisites, one `##` per goal with numbered steps, tables for
  reference, `## See also` with 2 to 4 pages.
- Troubleshooting entry: a `##` heading with the problem in the user's
  words or the exact message, then bold **Symptoms**, **Cause**, **Solution** (numbered) and **Prevention** when there
  is one.
