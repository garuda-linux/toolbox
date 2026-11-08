# User documentation

The user guide is an mdBook in `docs/src/`, built by
`scripts/build-docs.sh` (also the Cloudflare Pages build). Page titles and
their `docs/src/SUMMARY.md` entries stay in sync.

## Screenshots

- `docs/src/images/` holds copies of the dark kittest snapshots (Git LFS)
  after an `UPDATE_SNAPSHOTS=1` run: `<page>.png` from
  `<page>/page_dark.png`, `<page>-<name>.png` from `<page>/<name>_dark.png`.
- Exceptions: `boot` (`boot/menu`), `boot-repair` (`bootrepair/page`),
  `configs` (`configs/list`), `guide` (`guide/wide`), `home`
  (`shell/home`), `locales` (`locales/locales`), `network`
  (`network/status`), `services` (`services/menu`), `system-settings`
  (`tweaks/dropdown`), `users` (`users/users`), `diagnostics-upload`
  (`diagnostics/question`), `palette-open` (`shell/console_and_palette`).
- Pages that need pacman databases, `/etc/locale.gen`, GRUB or a user
  systemd bus take their shots from the `arch-selftest` artifacts. Shots
  of "not available" states belong only in `troubleshooting.md`.

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
  words or the exact message, then bold **Symptoms**, **Cause**,
  **Solution** (numbered) and **Prevention** when there is one.
