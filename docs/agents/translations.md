# Translations and Transifex

Transifex reads no Fluent. `cargo xtask i18n-export` writes
`app/gui/i18n/en-US/*.ftl` (the source of truth, never written by the
import) as Structured JSON to `i18n/garuda-toolbox_en.json`.
`cargo xtask i18n-import` writes `i18n/garuda-toolbox_<code>.json` back
into `app/gui/i18n/<locale>/*.ftl`. Both take `--check`.

## The conversion

- A locale folder maps to a Transifex code by its language (`de-DE` is
  `de`), with the region when two folders share one (`pt-BR` is `pt_BR`).
- A comment right above a message becomes its `developer_comment`.
- A CLDR plural becomes an ICU plural. A choice on other words becomes one
  entry per variant: Transifex shows only ICU plurals as plurals, an ICU
  select would reach translators as raw text.
- The export refuses attributes, terms, message references, functions,
  number literals, number keys, a non-plural choice inside a sentence and
  nested choices.
- The import refuses a variable the English text lacks and ICU that
  Fluent cannot hold (select, offsets, `=1` keys, a plural without
  `other`). Refused lines fail the task after the good ones are written.

## The import changes as little as it can

- An unchanged translation keeps its bytes.
- A translation equal to the English text never replaces an existing one,
  since Transifex fills untranslated strings with the source.
- A changed message gets the canonical shape and loses `# untranslated`.
- A missing message goes after the one before it in the English file.
- Export then import of the checked-in files changes nothing
  (`exporting_and_importing_the_real_files_changes_nothing`).

## CI

- `i18n-source` (default branch) commits a changed `i18n/`. Transifex
  pulls it from the repository.
- Transifex pushes to `translations_*` branches. `i18n-import` commits the
  Fluent files there, the shared `transifex-automerge` job merges. Every
  other job skips these branches.
- The automerge stage comes from the shared template. When it runs before
  `i18n-import`, the import's commit lands in the next pipeline.
