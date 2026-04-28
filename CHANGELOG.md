## [5.2.3] - 2026-04-28

### 🚀 Features

- *(setup-assistant)* Launch directly via home option; add to modules

### 🐛 Bug Fixes

- *(system-settings)* Correctly handle newline for garuda-update settings
- *(maintenance)* Correct typos
- Properly exit second instance and focus first
## [5.2.2] - 2026-04-24

### 🚀 Features

- *(migrations)* Introduce migration versions and migrate autostart file to new toolbox name/delete old
- *(system-settings)* Add Kontainer to options

### 🐛 Bug Fixes

- *(network-assistant)* Do not require root for toggling bluetooth/wifi
- *(system-settings)* Be specific about wpa_supplicant being a service
- *(system-settings)* Don't fail switching back to wpa_supplicant if one of the config fails to delete doesn't exist
- *(system-settings)* Garuda-update expects value set/not set and doesn't check for bool values; follow garuda-update variable rename
## [5.2.1] - 2026-04-19

### 🚀 Features

- *(system-settings)* Add garuda-update options

### 🐛 Bug Fixes

- *(boot-tools)* Use uuid-based boot entries, and correctly determine subentries for inclusion
- *(system-settings)* Correctly replace space separated default configs of garuda-update

### 📚 Documentation

- *(readme)* Update
## [5.2.0] - 2026-04-11

### 🚀 Features

- *(renderer)* Do not hardcode micro for root config management
- *(boot-tools)* Don't require password for reading GRUB settings; auto-prune GRUB tasks
- *(renderer)* Set language to system language if available and not overriden
- *(boot-tools)* Add editor for boot entries; never try to readFile via node as pkexec takes care of rootless read already
- *(gaming)* Add vesktop
- *(boot-tools)* Add plymouth preview; dract-rebuild on enable
- *(maintenance)* Distinguish between Arch/Chaotic-AUR mirrorlist refresh

### 🐛 Bug Fixes

- *(renderer)* Sanitize VISUAL env var from eventual signs causing issues
- *(renderer)* Sanitize all kind of braces via sanitizeEditor
- *(renderer)* Ensure terminal has the correct dimensions
- *(setup-assistant)* Correct linux-cachyos description; tasks to be done before showing success
- *(renderer)* Apply pending operations instantly starts executing
- *(terminal)* Another attempt at auto-focusing the terminal
- *(boot-tools)* Correctly set submenu boot entries as default
- *(maintenance)* Loop in remove orphans until all of them are gone

### ⚙️ Miscellaneous Tasks

- Sync package lists
## [5.1.2] - 2026-04-06

### 🐛 Bug Fixes

- Do not block input on cmd execute/resize
## [5.1.1] - 2026-04-04

### 🚀 Features

- *(renderer)* When both AUR and regular updates exist, offer buttons for both

### 🐛 Bug Fixes

- Revert to working electron-builder version
## [5.1.0] - 2026-04-04

### 🚀 Features

- *(renderer)* Blur termial background

### 🐛 Bug Fixes

- Don't report system updates twice
## [5.0.2] - 2026-04-04

### 🐛 Bug Fixes

- Tasks in terminal not executing correctly
- Properly reset directories in reset configs; use correct location for autostart

### 🎨 Styling

- Fix eslint/oxlint warnings and eslint error
- Fix eslint/oxlint warnings and eslint error; switch to oxfmt
## [5.0.1] - 2026-04-03

### 🐛 Bug Fixes

- Context menu blur not working in compiled mode; support all icon packs for icon lookups with better fallback
## [5.0.0] - 2026-04-03

### 🚀 Features

- *(renderer)* Add tooltip with descriptions for checkbox/system options
- *(renderer)* Hide dead and exited systemd services by default; allow toggling enabled
- Support real TTY shell output and writing back input
- Wrap remaining pkexec calls to profit from custom policykit prompt
- *(renderer)* Support romanian language
- *(renderer)* Implement package installation section of native setup assistant on first boot
- Argv function; start setup assistant via cmdline flag only; remove firstboot check and nvidia driver installation
- Display setup assistant success page after finishing tasks
- *(renderer)* Add config files module
- *(renderer)* Add smooth route transitions
- *(renderer)* Restore setup commands in setup assistant
- *(renderer)* Expand config files list
- Implement garuda-boot-options
- Implement garuda-boot-repair
- *(renderer)* Expand config files list; sort entries
- *(renderer)* Support config files directories; add some more
- Implement garuda-network-assistant
- *(renderer)* Add more tooltips for boot repair; adjust sizing/positioning of titles
- *(renderer)* Blur context menu; uniform look for diagnostics page
- *(renderer)* Integrate garuda-health in system-status on welcome screen and maintenance
- *(renderer)* Add garuda-health to diagnostics
- *(renderer)* Add command palette via ctrl + p
- *(renderer)* Uniform card based look for settings; network tools; config files

### 🐛 Bug Fixes

- Properly resolve app icons from custom themes
- Restore window bounds on Wayland
- Set window title and desktop name early to correctly report on Wayland
- Register IPC handlers before creating window to fix missing shell:execute IPC handler
- *(renderer)* Remove defer on gaming section; fixes jumping pictures; perf is good enough
- *(renderer)* Background image not always spanning full height; scrollbar appearing/disappearing moving content
- *(renderer)* Show boot options fields already when entering password
- *(renderer)* Blur background of dropdowns
- *(renderer)* Add back dropdown hints and restructure fields
- *(renderer)* Cards initially not blurred due to page transition; round corners equally in tablist
- Only send frontend logs via IPC if loglevel matches
- *(renderer)* Command palette not closing on ESC; use CSS variables for theming dropdown

### 💼 Other

- Use maximum compression

### 🚜 Refactor

- More broad support for custom icons in package lists; pull custom icons in gamer as well, if available
- *(renderer)* Put the further applications section into its own module for better visibility
- *(main)* Improve async handling in modules with Promise-based methods, optimize buffer flushing, and cleanup unused OS handlers

### ⚡ Performance

- Enable gpu acceleration and fix issue with vulkan on wayland
- *(renderer)* Show less items in paginated areas for less dom nodes, massively improving performance especially on additional packages tab

### 🎨 Styling

- Resolve eslint errors

### ⚙️ Miscellaneous Tasks

- Fix eslint config for newer versions
- *(flake.lock)* Update
- *(renderer)* Update search keywords with up-to-date keywords
- *(changelog)* Update
## [4.0.1] - 2026-03-09

### 🐛 Bug Fixes

- Attempt to fix autostart logic
- Do not apply autostart logic via effect

### ⚙️ Miscellaneous Tasks

- Remove firstboot logic
## [4.0.0] - 2026-03-07

### 🚀 Features

- *(packages)* Improve arch wiki parsing script; enricht with appstream data; update lists
- *(packages)* Add appicon module, to resolve icons from different sources and display them in the app lists
- *(settings)* Add home config module and possibility of changing settings/configs with 2 initial configs
- *(migration)* Safely migrate user configs
- *(main)* Safeguard against running as root
- *(migration)* Migrate plasma dock to new desktop entry

### 🐛 Bug Fixes

- *(i18n)* Add missing translations for new os-interact options
- Drop webgl xterm addon as it was causing issues; resolve deprecations
- *(configs)* Properly determine home dir; show configs conditionally based on desktop environment
- *(main)* Allow app-icon protocol via CSP to fix non-rendering icons
- *(packages)* Properly reset pagination to 1 in case of tab change
- Exclude AUR packages from lists by default

### 🚜 Refactor

- Rebrand to Garuda Toolbox
- *(renderer)* Apply angular migrations for modern syntax

### 📚 Documentation

- *(changelog)* Update
## [3.5.2] - 2026-01-03

### 🚀 Features

- *(ui)* Added border and button hightlight on hover on cards for good user interactivity in maintenance page and fixed tab border-bottom responsive issue
- *(ui)* Added border and button hightlight on hover on cards for good user interactivity in maintenance page and fixed tab border-bottom responsive issue

### ⚙️ Miscellaneous Tasks

- *(i18n)* Pull Transifex translations
- *(i18n)* Pull Transifex translations
- *(i18n)* Pull Transifex translations
## [3.5.1] - 2025-11-02

### 🐛 Bug Fixes

- *(first-boot)* Hide UI on firstboot procedure

### ⚙️ Miscellaneous Tasks

- Further cleanup package lists
## [3.5.0] - 2025-11-01

### 🚀 Features

- *(ui)* Added border on hover on cards for good user interactivity in welcome page

### ⚙️ Miscellaneous Tasks

- *(config)* Migrate config renovate.json
- *(i18n)* Pull Transifex translations
- Cleanup package lists
## [3.4.0] - 2025-10-26

### 🚀 Features

- *(renderer)* Better information about direct action failures, pass down errors
- *(renderer)* Introduce new refresh keyring action and be more specific about garuda-update remote actions
- *(renderer)* Derive more colors from currently active CSS for better custom theme support

### 🐛 Bug Fixes

- *(renderer)* Refresh mirrorlist not opening reflector-simple
- *(renderer)* Show keyring refresh in the terminal, add missing sudo

### ⚙️ Miscellaneous Tasks

- Add corepack as this is no longer shipped in Node 25
- Cleanup no more in-repo packages
## [3.3.1] - 2025-10-08

### 🚀 Features

- *(shell.nix)* Replace with actual working shell.nix

### 🐛 Bug Fixes

- *(splash)* Do not limit application size to 100% screen size

### 📚 Documentation

- *(changelog)* Update
## [3.3.0] - 2025-10-03

### 🚀 Features

- *(i18n)* Bring pt-BR support
- *(i18n)* Support uk, translation by devil6a

### 🐛 Bug Fixes

- *(renderer)* Parse and check /etc/shells for valid shells / paths
- *(i18n)* Typo
- *(main)* ModuleRunner not being awaited, strip unnecessary things

### ⚡ Performance

- Lazy-load components, spawn window first and show a static splash screen until Angular is ready

### ⚙️ Miscellaneous Tasks

- Drop obsolete packages from lists
## [3.2.0] - 2025-09-19

### 🚀 Features

- *(i18n)* Implement transifex with automerge
- *(i18n)* Add ukrainian
- *(i18n)* Add et
- *(renderer)* Resize terminal automatically
- *(renderer)* Do not depend on garuda-update for system updates

### 🐛 Bug Fixes

- *(renderer)* Do not show kernel duplicates in case of multiple repos having them
- Return to working electron-builder version
- *(renderer)* Do not use progress bars in all places
- Lockfile

### ⚙️ Miscellaneous Tasks

- Drop obsolete package from lists
- *(i18n)* Pull Transifex translations
- *(i18n)* Pull Transifex translations
- *(i18n)* Pull Transifex translations
## [3.1.5] - 2025-08-13

### 🐛 Bug Fixes

- *(renderer)* Visual regressions due to Angular 20/primeng api changes
- *(renderer)* Drop obsolete packages, change duckstation -> gpl
- *(renderer)* Re-add the no longer broken fa-icons
- *(renderer)* Try to trigger cdr manually in case of non-appearing update counts
- *(renderer)* Properly retrigger change detection
## [3.1.4] - 2025-07-31

### 🐛 Bug Fixes

- Properly use launch-terminal, fix garuda-chroot launch
## [3.1.3] - 2025-07-29

### 🐛 Bug Fixes

- *(renderer)* Do not attempt to run setup assistant if in live session
- *(main)* Do not set no_new_privs when re-launching application

### ⚙️ Miscellaneous Tasks

- *(main)* Remove unnecessary ApplicationTerminatorOnLastWindowClose
## [3.1.2] - 2025-07-28

### 🚀 Features

- Migrate to Angular 20.1, new themes package, vo1ded themes
- Switch to @garudalinux shell component

### 🐛 Bug Fixes

- Remaining angular 20/theme package migration issues
- *(home)* Launch calamares installer with pkexec-gui instead

### 📚 Documentation

- *(changelog)* Update

### 🎨 Styling

- Prevent formatting pnpm-lock.yaml

### ⚙️ Miscellaneous Tasks

- Bring angular-eslint, fixup errors and correctly enable in pre-commit
- Cache pnpm store
- Fix pnpm i oom due to node 24 bug
## [3.1.1] - 2025-07-26

### 🐛 Bug Fixes

- *(first-boot)* Properly handle first boot scenario
- *(renderer)* Removed random weird - sign on the bottom of the page
- *(gaming)* Changed luxtorpeda to the currently in-repo version
## [3.1.0] - 2025-07-20

### 🚀 Features

- *(added-tootips-to-games-section)* Wrote a small script to pull package descriptions. Added those descriptions as tooltips in the various packages in the gaming sections
- Save and restore window state
- *(renderer)* Add refresh button to system status

### 🐛 Bug Fixes

- *(renderer)* Do not source bashrc or profile when calling bash
- *(renderer)* Bring back reaction to auto-start toggle, handle it properly even with Tauri-created files

### 📚 Documentation

- *(readme)* Update

### ⚙️ Miscellaneous Tasks

- Move script to assets, shfmt
- Drop non-existant packages from lists
- Use reusable job templates and add GitHub close PR workflow
## [3.0.2] - 2025-06-29

### 🐛 Bug Fixes

- *(renderer)* Context menu not working (and looking out of design)
- *(designer)* Remove unnecessary optional chain operators

### 📚 Documentation

- *(changelog)* Update

### ⚙️ Miscellaneous Tasks

- Add renovate bot
- Drop .npmrc and explicitly pass --shamefully-hoist on PKGBUILD installs
- Drop per-repo renovate on favor of org-wide setup
## [3.0.1] - 2025-06-29

### 🚀 Features

- *(task-manager)* Add timeout option with none by default and localized messages for script execution, add task finished message

### 🐛 Bug Fixes

- *(maintenance)* Reset not going through due to resolve being used
- *(terminal)* Correct progress calculation for single-task scenarios

### 📚 Documentation

- *(changelog)* Update
## [3.0.0] - 2025-06-28

### 🚀 Features

- Implement spawn streaming via ipc
- Have spawn streaming actually work
- *(renderer)* Native context menu
- *(logging)* Substantially improve logging
- Logging revamp, fixes preload issues occurred with electron-timber
- *(search)* Add initial app search
- Add more translations to .desktop file
- *(theming)* Add PrimeNG designer directly in-app
- *(wallpaper)* Implement customizable background settings and blur effects
- *(designer)* Integrate custom theme support for terminals and scrollbar color management
- Reimplement http calls in main process, fixes privatebin uploads
- *(designer)* Add Dr460nized theme presets and enhance theme creation workflow

### 🐛 Bug Fixes

- Missing dep
- Use vite for frontend build
- Properly included all deps
- Issues with main assets
- Devmode
- *(types)* Make types work
- *(renderer)* No absolute paths in prod mode
- Only include dist files of workspace packes, correct app name
- *(store)* Don't use invalid context
- Import via ESM module syntax and not glob
- Add back required commands
- *(logging)* Make some overly verbose logs debug/trace
- *(packages)* Properly filter out non-available packages
- *(store)* Overlapping function/variable name
- *(logging)* Renderer logging now only goes via console
- *(readFile)* Adapt to new assets dir, no more symlinks
- *(renderer)* Apply operations button, window operations
- Made the apply button shine again, minimum window size
- Some weird graphics, broken spawn shell emitters
- Devtools for testing pkg availability, updated pipelines
- *(settings)* Don't show select dropdown under panel, use primeng slider
- *(designer)* Improve flow
- *(renderer)* We suddenly have index.html in a new client folder
- *(designer)* Get back strict type-checking (what the fuck is this designer code)
- *(shell)* Allow last command, needed for first boot check

### 💼 Other

- Skip publish step on tags

### 🚜 Refactor

- Current electron refactor progress
- Move ipcRenderer.invoke calls to preload function exports

### 📚 Documentation

- Update, add NixOS notice

### 🎨 Styling

- Reformat with line length 120
- Reformat with the right settings, make treefmt use the right one

### ⚙️ Miscellaneous Tasks

- Update electron-builder config
- Cleanup and new commit for new build
- Bump
- Bump deps
- Cleanup
## [2.5.1] - 2025-05-24

### 🚀 Features

- *(terminal)* Allow printing scripts to run prior to running

### 🐛 Bug Fixes

- *(kernels)* Toggling an installed kernel without headers installing headers
- *(packages)* Remove invalid packages or those moved to AUR
- *(window)* Resize both dimensions
- *(kernels)* Exception on undefined kernelVersion, causing loading to never finish
- *(renderer)* Set WEBKIT_DISABLE_DMABUF_RENDERER always. Byebye blur and good looking aesthetics.

### 💼 Other

- *(debug)* Add CrabNebula devtools for more useful information

### 📚 Documentation

- *(changelog.md)* Update

### 🧪 Testing

- Add some first working unit tests

### ⚙️ Miscellaneous Tasks

- *(package.json)* Updates
- *(kernels)* Add some trace logging
- Update dependencies
- *(packages)* Remove non-existing packages
- Update more deps
## [2.5.0] - 2025-03-30

### 🚀 Features

- *(translations)* Massively improve by leveraging Gemini 2.5 Pro to translate the full JSON

### 🐛 Bug Fixes

- *(language)* First time changing language not setting it
- *(garuda-update)* Only allow running remote commands directly
- *(task-manager)* ExecuteAndWaitTerminal not waiting

### 📚 Documentation

- *(changelog)* Update
## [2.4.0] - 2025-03-29

### 🚀 Features

- *(locales)* Add new component allowing language changes
- *(packages)* Safeguard against non-available pkgnames
- *(theming)* More design choices
- *(maintenance)* Prompt to delete merged .pacnew, make snapshot success mandatory
- *(window)* Don't hide buttons on systems without the Kwin setting active
- Make things more navigatable, preparement for cmd args
- *(cli)* Allow CLI commands to open tabs, pages and execute diagnostic actions
- *(configService)* Refresh pkgs, locale and reboot pending after actions
- *(terminal)* Add copy, upload to privatebin buttons, keyboard shortcuts
- *(app)* Allow closing globally via ctrl + q
- *(system-settings)* Add option to choose iwd as NetworkManager backend
- Migrate to nx
- Verbose cmdarg, fix diagnostics args

### 🐛 Bug Fixes

- *(angular)* Increase budgets
- *(system-components)* Correct systemd service name for opensnitch
- *(system-settings)* Correct 'service' to 'pkg' for "Intel-undervolt installed" checkbox
- *(system-settings)* Correct 'service' to 'pkg' for "Thermald installed" checkbox
- *(package-lists)* Remove xbox-generic-controller from gaming
- *(theming)* Make checkboxes easier to see, misc fixes
- *(maintenance)* Make it clear how to merge pacdiff, drop kompare, warn about destructiveness
- *(kernels)* Update kernel state after kernels initialized for the first time
- *(maintenance)* Pacdiff file not being deleted after y
- *(kernels)* Distinguish broken and non-installed modules
- *(configService)* Too many update executions
- *(http)* Downgrade tauri http plugin, adapter currently breaks actions
- *(kernels)* Remove unnecessary double call
- *(translations)* Missing translations with angular optimization
- Focus active window when opening rani a second time

### 🚜 Refactor

- Move status data to cached service, allow running external AUR update

### 🎨 Styling

- *(prettier)* Set tailwindcss plugin active and reformat

### ⚙️ Miscellaneous Tasks

- Post nx-migration cleanup invalid tests
- Set nx defaults for generation of components
## [2.3.0] - 2025-03-21

### 🚀 Features

- *(packages)* Use searchable table instead for display
- *(system-tools)* Only load components when tab has been selected
- *(kernels)* Add kernels component
- *(kernels)* Add dkms module check and loading indicator
- *(gaming)* Add more WINE versions
- *(language-packs)* New component
- *(settings)* Move to dedicated page
- *(maintenance)* Allow merging pacdiff files interactively
- *(system-status)* Warn about pending reboot
- *(system-settings)* Add bpftune

### 🐛 Bug Fixes

- *(system-components)* Scanning-support never being enabled
- *(os-interact)* Check for systemd sockets as well, fixing non-recognized sockets
- *(pnpm-lock.yaml)* Sync with package.json
- *(config-service)* Apply Loglevel on settings change
- *(notifications)* Unbreak not-sending notifications

### 🚜 Refactor

- Massively improve performance by caching and using services
- Use pacman regex for prefiltering, print commands used

### 📚 Documentation

- *(changelog)* Update

### ⚙️ Miscellaneous Tasks

- Drop ununsed code
- Fix commitizen installation
## [2.2.0] - 2025-03-18

### 🚀 Features

- *(system-settings)* Add packages section
- *(translation)* Use inbuilt Tauri resources to load translations
- Version 2.2.0

### 🐛 Bug Fixes

- *(system-components)* Virtualbox checking for group rather than pkg
- *(packages)* Invalid pkgnames, fixed parser as well
- *(language)* Only update language once on startup
- *(translations)* Safeguard for failing to load translation files
## [2.1.0] - 2025-03-18

### 🚀 Features

- *(garuda-update)* Set GARUDA_UPDATE_RANI=1 to notify garuda-update of usage via rani

### 🐛 Bug Fixes

- *(vmware,virtualbox)* Disable DMABUF renderer
- *(lib.rs)* Attempt fixing detection by making strings lowercase
- *(dynamic-checkboxes)* Allow disabled entries to be properly managed.

### 📚 Documentation

- *(changelog)* Add changelog

### ⚙️ Miscellaneous Tasks

- *(flake.lock)* Update
## [2.0.0] - 2025-03-17

### 🚀 Features

- *(window)* Responsive window button states
- *(loadingservice)* Use refcount on loading indicator
- *(font)* Use system-ui
- *(init)* Resize window to reasonable size when monitor is too small
- Move parseArchWiki script to Typescript
- *(system-status)* Report AUR updates too
- *(system-status)* Support schedule AUR updates, update on task updates
- Strip AUR functionality, we decided to not support it
- *(system-components)* Expand with containers section
- Menu bar "terminal" button glows actively when tasks are pending
- *(terminal)* Only show buttons when needed/reasonable

### 🐛 Bug Fixes

- *(os-interact)* Fix setting dns to default
- *(diagnostics)* Disable logging symbol after loading full diagnostics
- *(system-status)* Invalid update versions
- *(angular)* Zoneless change detection complaining
- *(gamer)* Some icons being improperly sized
- *(gaming)* Missing pacakges, add script to check for missing ones
- *(theme)* Light theme not working, wrong window buttons on the right
- *(window)* Set reasonable menu breakpoint, set logicalSize dynamically
- *(dynamic-checkboxes)* Ensure already enabled/installed packages are not disabled
- Duplicated updates, duplicated logic
- *(systemd-services)* Event reporting failure due to no output
- Menu bar "terminal" button size changing when glow effect is active

### 🎨 Styling

- Declare types as types

### ⚙️ Miscellaneous Tasks

- Cleanup unused
- Add package list check
- Also check AUR packages
- Fix missing pnpm, less verbose script
## [1.3.1] - 2025-03-12

### 🐛 Bug Fixes

- Set WEBKIT_DISABLE_COMPOSITING_MODE for NVIDIA GPUs
## [1.3.0] - 2025-03-06

### 🐛 Bug Fixes

- *(package.json,cargo.toml)* Cleanup required packages
- *(translation)* Load translations before the application loads
- *(home)* Utilize launch-terminal garuda-libs script for chroot
## [1.2.0] - 2025-03-05

### 🚀 Features

- Allow confirmation-less app exit when there are no pending tasks.
- *(operation-manager)* Do not store/restore pending operations when exiting app
- *(logs)* Allow setting loglevel

### 🐛 Bug Fixes

- *(shell.nix)* Fix nix build env
- *(style)* Disable any kind of text selection
- *(diagnostics)* Do not use garuda-inxi funstuff
- *(config,home)* Fix excessive redraws, live system detection, angular change detection
- More efficient file existence checks via plugin-fs
- *(status)* Logs reporting update check failure when none are availble
- *(theme-handler)* Use updateConfig method to set new setting
- *(xterm)* Use computed signal to prevent wrong theme

### 🚜 Refactor

- *(first-boot)* Revamp first boot checks
- *(darkmode)* Redo darkmode to be more consistent
- *(language-switcher)* Redo language-switcher to be more consistent
- *(app)* Pre-init config before rendering application
- *(app)* More reliable menu/label setup, more reliable transloco
## [1.1.0] - 2025-03-02

### 🚀 Features

- Run setup-assistant on first boot, relaunch after update
- *(gaming)* Import new icons from gamer, placeholders update
## [1.0.2] - 2025-03-02

### 🚀 Features

- *(welcome)* Don't show status on live system

### 🐛 Bug Fixes

- No max size, inform about errors, calamares not starting
## [1.0.1] - 2025-03-02

### 🐛 Bug Fixes

- User set to null, loglevel to info, skip mirrorlist by default
## [1.0.0] - 2025-03-02

### 🚀 Features

- Safeguards app shutdown/remove action, only safe undone
- *(operations)* Allow aborting running, and running directly
- *(system-status)* Warn on >2w no update
- Add autostart config, remove problematic vboxkvm

### 🐛 Bug Fixes

- *(nvidia)* Attempt disabling dmabuf renderer
- *(gaming)* Invalid parsed entries
- *(operation-manager)* Passed signal instead of value
- *(diagnostics)* Make buttons not rounded, comply with rest of app

### ⚙️ Miscellaneous Tasks

- Extend Arch wiki list parsing
## [0.2.0] - 2025-03-01

### 🚀 Features

- *(privilege-manager)* Keep one-time use creds between a run of multiple
- Make settings menu entries dynamic, use fontawesome icons
- Improve visuals on home and other places
- Set reasonable min window size, report progress, misc fixes
- *(gaming)* Derive list of games from AUR, check whether available
- *(operations)* Sort pending list after order, cleanup subscriptions
- Use cdr onpush for better performance, support installing AUR games
- Support reporting pacdiff/update status on home, allow ensuring package is installed

### 🐛 Bug Fixes

- Save state on shutdown
- *(system-settings)* Toggling hblock now works as expected
- Singleton ConfigService, shell not initializing
- *(privilege-manager)* Handle aborted input gracefully
- *(checkboxes)* Fix unexpected toggle behaviour
- *(diagnostics)* Page not loading due to privatebinclient constructor missing
- *(deps)* Add missing dep, remove obsolete
- *(window)* Add missing permission
- Hblock enable action, don't try translating game titles
- Use better icons for status

### 🚜 Refactor

- *(configService)* Move user determination to configService
- *(configService)* Source darkMode from configService

### 📚 Documentation

- Add deps, run funstuff

### ⚙️ Miscellaneous Tasks

- Add linting
- *(types)* Declare a handful interfaces as types
- Version 0.2.0
## [0.1.1] - 2025-02-27

### 🚀 Features

- *(terminal)* Use WebGlAddon

### 🐛 Bug Fixes

- *(gamer)* No more broken icons
- *(privilege-manager)* Don't dump password into logs
## [0.1.0] - 2025-02-27

### 🚀 Features

- *(logger)* Introduce custom logger class for loglevels
- *(translations)* Add trashy auto-translated languages
- *(shortcuts)* Prevent default browser shortcuts in prod

### 🐛 Bug Fixes

- *(translations)* Disable script optimisations in prod mode

### ⚙️ Miscellaneous Tasks

- Version 0.1.0
## [0.0.1] - 2025-02-27

### 🚀 Features

- Initial commit
- *(gamer)* Finished queue logic, much more entries
- Systemd services actions/comp, reset configs, many improvements
- Integrate btrfs-assistant and apps in need of sudo
- Correct icons, provide some services globally
- Fake window buttons
- Start porting sys components, misc improvements
- A boat load of improvements, too many to specify
- Saving progress, loading indicator, tons of improvements
- Heavy refactoring to make use of privilege/operation services, add loading indicator
- Make more use of loadingService, diagnostics refactor
- Tabbed maintenance, skip done, support live/installed actions in welcome, theme fixes
- Adjust package name, reformat

### 🐛 Bug Fixes

- *(privilege-manager)* Wrong authenticated status
- Operation management, show disabled services
- Direct run should open terminal
- Await store being ready, make initial window higher, readme link

### 🚜 Refactor

- Migrate to configService

### 📚 Documentation

- *(readme)* Add

### ⚙️ Miscellaneous Tasks

- Drop some unused code, update cargo deps
- Cleanup unused code
- Add some ci
- Use non-alpine container, add missing libsoup
- Update mirrorlist before installation
- Limit to bundle directories
- Add the regular checks, try bullseye
- Flake.nix setup
- Add .desktop file
- Only upload the needed files
