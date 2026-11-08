# The Languages page: the locales of /etc/locale.gen, the system language
# and the language packs of installed programs, all queued for Apply.

page-locales-subtitle = The languages, number and date formats this system can use, and translations for your programs.
locales-refresh = Read again
locales-tabs = Languages tabs
locales-tab-locales = Locales
locales-tab-packs = Language packs
locales-queued = Changes with Apply

locales-search = Search locales
locales-search-hint = Search locales…
locales-filter-utf8 = UTF-8 only
locales-filter-enabled = Enabled only
locales-list = Locales
locales-reading = Reading the locales…
locales-none = No locale matches
locales-none-detail = The search looks at the names of languages, countries, locales and character sets.
locales-clear-search = Clear search
locales-unavailable = Locales are not available on this system
locales-unavailable-detail = This system does not generate its locales from /etc/locale.gen.
locales-enabled = Enabled
locales-disabled = Disabled
locales-title-territory = { $language } ({ $territory })
locales-detail = { $code }, { $charset }
locales-detail-system = { $code }, { $charset }, system language

locales-system = System language
locales-system-detail = The system language takes effect at the next login. Only enabled locales are offered: enable a locale in the list below first.
locales-system-unset = Not set
locales-system-choice = { $language }, { $code }
locales-system-choice-territory = { $language } ({ $territory }), { $code }

# Why a click was not queued.
locales-refused-system-locale = { $locale } is the system language. Choose another system language before you disable it.
locales-refused-last-locale = At least one locale must stay enabled.
locales-refused-not-enabled = { $locale } is not enabled. Enable the locale first.
locales-dismiss = Dismiss

locales-packs-intro = Language packs for your enabled locales and installed programs. Missing: { $count }
locales-packs-intro-plain = Language packs for your enabled locales and installed programs.
locales-packs-add-missing = Queue all missing
locales-packs-sort = Sort language packs
locales-packs-sort-package = Package
locales-packs-sort-program = Program
locales-packs-sort-language = Language
locales-packs-sort-selected = Selected first
locales-packs-list = Language packs
locales-packs-reading = Reading the language packs…
locales-packs-none = No language packs found
locales-packs-none-detail = No installed program has a language pack for an enabled locale.
locales-packs-unavailable = Language packs are not available on this system
locales-packs-unavailable-detail = The package databases of pacman could not be read.
locales-pack-installed = Installed
locales-pack-missing = Not installed
locales-pack-detail = For { $base }: { $locale }
