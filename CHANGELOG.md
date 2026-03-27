## [unreleased]

### 🚀 Features

- _(renderer)_ Add tooltip with descriptions for checkbox/system options
- _(renderer)_ Hide dead and exited systemd services by default; allow toggling enabled
- Support real TTY shell output and writing back input
- Wrap remaining pkexec calls to profit from custom policykit prompt

### 🐛 Bug Fixes

- Properly resolve app icons from custom themes
- Restore window bounds on Wayland
- Set window title and desktop name early to correctly report on Wayland
- Register IPC handlers before creating window to fix missing shell:execute IPC handler

### 💼 Other

- Use maximum compression

### ⚡ Performance

- Enable gpu acceleration and fix issue with vulkan on wayland
- _(renderer)_ Show less items in paginated areas for less dom nodes, massively improving performance especially on additional packages tab

### ⚙️ Miscellaneous Tasks

- Fix eslint config for newer versions
- _(flake.lock)_ Update
- _(renderer)_ Update search keywords with up-to-date keywords

## [4.0.1] - 2026-03-09

### 🐛 Bug Fixes

- Attempt to fix autostart logic
- Do not apply autostart logic via effect

### ⚙️ Miscellaneous Tasks

- Remove firstboot logic

## [4.0.0] - 2026-03-07

### 🚀 Features

- _(packages)_ Improve arch wiki parsing script; enricht with appstream data; update lists
- _(packages)_ Add appicon module, to resolve icons from different sources and display them in the app lists
- _(settings)_ Add home config module and possibility of changing settings/configs with 2 initial configs
- _(migration)_ Safely migrate user configs
- _(main)_ Safeguard against running as root
- _(migration)_ Migrate plasma dock to new desktop entry

### 🐛 Bug Fixes

- _(i18n)_ Add missing translations for new os-interact options
- Drop webgl xterm addon as it was causing issues; resolve deprecations
- _(configs)_ Properly determine home dir; show configs conditionally based on desktop environment
- _(main)_ Allow app-icon protocol via CSP to fix non-rendering icons
- _(packages)_ Properly reset pagination to 1 in case of tab change
- Exclude AUR packages from lists by default

### 🚜 Refactor

- Rebrand to Garuda Toolbox
- _(renderer)_ Apply angular migrations for modern syntax

### 📚 Documentation

- _(changelog)_ Update

## [3.5.2] - 2026-01-03

### 🚀 Features

- _(ui)_ Added border and button hightlight on hover on cards for good user interactivity in maintenance page and fixed tab border-bottom responsive issue
- _(ui)_ Added border and button hightlight on hover on cards for good user interactivity in maintenance page and fixed tab border-bottom responsive issue

### ⚙️ Miscellaneous Tasks

- _(i18n)_ Pull Transifex translations
- _(i18n)_ Pull Transifex translations
- _(i18n)_ Pull Transifex translations

## [3.5.1] - 2025-11-02

### 🐛 Bug Fixes

- _(first-boot)_ Hide UI on firstboot procedure

### ⚙️ Miscellaneous Tasks

- Further cleanup package lists

## [3.5.0] - 2025-11-01

### 🚀 Features

- _(ui)_ Added border on hover on cards for good user interactivity in welcome page

### ⚙️ Miscellaneous Tasks

- _(config)_ Migrate config renovate.json
- _(i18n)_ Pull Transifex translations
- Cleanup package lists

## [3.4.0] - 2025-10-26

### 🚀 Features

- _(renderer)_ Better information about direct action failures, pass down errors
- _(renderer)_ Introduce new refresh keyring action and be more specific about garuda-update remote actions
- _(renderer)_ Derive more colors from currently active CSS for better custom theme support

### 🐛 Bug Fixes

- _(renderer)_ Refresh mirrorlist not opening reflector-simple
- _(renderer)_ Show keyring refresh in the terminal, add missing sudo

### ⚙️ Miscellaneous Tasks

- Add corepack as this is no longer shipped in Node 25
- Cleanup no more in-repo packages

## [3.3.1] - 2025-10-08

### 🚀 Features

- _(shell.nix)_ Replace with actual working shell.nix

### 🐛 Bug Fixes

- _(splash)_ Do not limit application size to 100% screen size

### 📚 Documentation

- _(changelog)_ Update

## [3.3.0] - 2025-10-03

### 🚀 Features

- _(i18n)_ Bring pt-BR support
- _(i18n)_ Support uk, translation by devil6a

### 🐛 Bug Fixes

- _(renderer)_ Parse and check /etc/shells for valid shells / paths
- _(i18n)_ Typo
- _(main)_ ModuleRunner not being awaited, strip unnecessary things

### ⚡ Performance

- Lazy-load components, spawn window first and show a static splash screen until Angular is ready

### ⚙️ Miscellaneous Tasks

- Drop obsolete packages from lists

## [3.2.0] - 2025-09-19

### 🚀 Features

- _(i18n)_ Implement transifex with automerge
- _(i18n)_ Add ukrainian
- _(i18n)_ Add et
- _(renderer)_ Resize terminal automatically
- _(renderer)_ Do not depend on garuda-update for system updates

### 🐛 Bug Fixes

- _(renderer)_ Do not show kernel duplicates in case of multiple repos having them
- Return to working electron-builder version
- _(renderer)_ Do not use progress bars in all places
- Lockfile

### ⚙️ Miscellaneous Tasks

- Drop obsolete package from lists
- _(i18n)_ Pull Transifex translations
- _(i18n)_ Pull Transifex translations
- _(i18n)_ Pull Transifex translations

## [3.1.5] - 2025-08-13

### 🐛 Bug Fixes

- _(renderer)_ Visual regressions due to Angular 20/primeng api changes
- _(renderer)_ Drop obsolete packages, change duckstation -> gpl
- _(renderer)_ Re-add the no longer broken fa-icons
- _(renderer)_ Try to trigger cdr manually in case of non-appearing update counts
- _(renderer)_ Properly retrigger change detection

## [3.1.4] - 2025-07-31

### 🐛 Bug Fixes

- Properly use launch-terminal, fix garuda-chroot launch

## [3.1.3] - 2025-07-29

### 🐛 Bug Fixes

- _(renderer)_ Do not attempt to run setup assistant if in live session
- _(main)_ Do not set no_new_privs when re-launching application

### ⚙️ Miscellaneous Tasks

- _(main)_ Remove unnecessary ApplicationTerminatorOnLastWindowClose

## [3.1.2] - 2025-07-28

### 🚀 Features

- Migrate to Angular 20.1, new themes package, vo1ded themes
- Switch to @garudalinux shell component

### 🐛 Bug Fixes

- Remaining angular 20/theme package migration issues
- _(home)_ Launch calamares installer with pkexec-gui instead

### 📚 Documentation

- _(changelog)_ Update

### 🎨 Styling

- Prevent formatting pnpm-lock.yaml

### ⚙️ Miscellaneous Tasks

- Bring angular-eslint, fixup errors and correctly enable in pre-commit
- Cache pnpm store
- Fix pnpm i oom due to node 24 bug

## [3.1.1] - 2025-07-26

### 🐛 Bug Fixes

- _(first-boot)_ Properly handle first boot scenario
- _(renderer)_ Removed random weird - sign on the bottom of the page
- _(gaming)_ Changed luxtorpeda to the currently in-repo version

## [3.1.0] - 2025-07-20

### 🚀 Features

- _(added-tootips-to-games-section)_ Wrote a small script to pull package descriptions. Added those descriptions as tooltips in the various packages in the gaming sections
- Save and restore window state
- _(renderer)_ Add refresh button to system status

### 🐛 Bug Fixes

- _(renderer)_ Do not source bashrc or profile when calling bash
- _(renderer)_ Bring back reaction to auto-start toggle, handle it properly even with Tauri-created files

### 📚 Documentation

- _(readme)_ Update

### ⚙️ Miscellaneous Tasks

- Move script to assets, shfmt
- Drop non-existant packages from lists
- Use reusable job templates and add GitHub close PR workflow

## [3.0.2] - 2025-06-29

### 🐛 Bug Fixes

- _(renderer)_ Context menu not working (and looking out of design)
- _(designer)_ Remove unnecessary optional chain operators

### 📚 Documentation

- _(changelog)_ Update

### ⚙️ Miscellaneous Tasks

- Add renovate bot
- Drop .npmrc and explicitly pass --shamefully-hoist on PKGBUILD installs
- Drop per-repo renovate on favor of org-wide setup

## [3.0.1] - 2025-06-29

### 🚀 Features

- _(task-manager)_ Add timeout option with none by default and localized messages for script execution, add task finished message

### 🐛 Bug Fixes

- _(maintenance)_ Reset not going through due to resolve being used
- _(terminal)_ Correct progress calculation for single-task scenarios

### 📚 Documentation

- _(changelog)_ Update

## [3.0.0] - 2025-06-28

### 🚀 Features

- Implement spawn streaming via ipc
- Have spawn streaming actually work
- _(renderer)_ Native context menu
- _(logging)_ Substantially improve logging
- Logging revamp, fixes preload issues occurred with electron-timber
- _(search)_ Add initial app search
- Add more translations to .desktop file
- _(theming)_ Add PrimeNG designer directly in-app
- _(wallpaper)_ Implement customizable background settings and blur effects
- _(designer)_ Integrate custom theme support for terminals and scrollbar color management
- Reimplement http calls in main process, fixes privatebin uploads
- _(designer)_ Add Dr460nized theme presets and enhance theme creation workflow

### 🐛 Bug Fixes

- Missing dep
- Use vite for frontend build
- Properly included all deps
- Issues with main assets
- Devmode
- _(types)_ Make types work
- _(renderer)_ No absolute paths in prod mode
- Only include dist files of workspace packes, correct app name
- _(store)_ Don't use invalid context
- Import via ESM module syntax and not glob
- Add back required commands
- _(logging)_ Make some overly verbose logs debug/trace
- _(packages)_ Properly filter out non-available packages
- _(store)_ Overlapping function/variable name
- _(logging)_ Renderer logging now only goes via console
- _(readFile)_ Adapt to new assets dir, no more symlinks
- _(renderer)_ Apply operations button, window operations
- Made the apply button shine again, minimum window size
- Some weird graphics, broken spawn shell emitters
- Devtools for testing pkg availability, updated pipelines
- _(settings)_ Don't show select dropdown under panel, use primeng slider
- _(designer)_ Improve flow
- _(renderer)_ We suddenly have index.html in a new client folder
- _(designer)_ Get back strict type-checking (what the fuck is this designer code)
- _(shell)_ Allow last command, needed for first boot check

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

- _(terminal)_ Allow printing scripts to run prior to running

### 🐛 Bug Fixes

- _(kernels)_ Toggling an installed kernel without headers installing headers
- _(packages)_ Remove invalid packages or those moved to AUR
- _(window)_ Resize both dimensions
- _(kernels)_ Exception on undefined kernelVersion, causing loading to never finish
- _(renderer)_ Set WEBKIT_DISABLE_DMABUF_RENDERER always. Byebye blur and good looking aesthetics.

### 💼 Other

- _(debug)_ Add CrabNebula devtools for more useful information

### 📚 Documentation

- _(changelog.md)_ Update

### 🧪 Testing

- Add some first working unit tests

### ⚙️ Miscellaneous Tasks

- _(package.json)_ Updates
- _(kernels)_ Add some trace logging
- Update dependencies
- _(packages)_ Remove non-existing packages
- Update more deps

## [2.5.0] - 2025-03-30

### 🚀 Features

- _(translations)_ Massively improve by leveraging Gemini 2.5 Pro to translate the full JSON

### 🐛 Bug Fixes

- _(language)_ First time changing language not setting it
- _(garuda-update)_ Only allow running remote commands directly
- _(task-manager)_ ExecuteAndWaitTerminal not waiting

### 📚 Documentation

- _(changelog)_ Update

## [2.4.0] - 2025-03-29

### 🚀 Features

- _(locales)_ Add new component allowing language changes
- _(packages)_ Safeguard against non-available pkgnames
- _(theming)_ More design choices
- _(maintenance)_ Prompt to delete merged .pacnew, make snapshot success mandatory
- _(window)_ Don't hide buttons on systems without the Kwin setting active
- Make things more navigatable, preparement for cmd args
- _(cli)_ Allow CLI commands to open tabs, pages and execute diagnostic actions
- _(configService)_ Refresh pkgs, locale and reboot pending after actions
- _(terminal)_ Add copy, upload to privatebin buttons, keyboard shortcuts
- _(app)_ Allow closing globally via ctrl + q
- _(system-settings)_ Add option to choose iwd as NetworkManager backend
- Migrate to nx
- Verbose cmdarg, fix diagnostics args

### 🐛 Bug Fixes

- _(angular)_ Increase budgets
- _(system-components)_ Correct systemd service name for opensnitch
- _(system-settings)_ Correct 'service' to 'pkg' for "Intel-undervolt installed" checkbox
- _(system-settings)_ Correct 'service' to 'pkg' for "Thermald installed" checkbox
- _(package-lists)_ Remove xbox-generic-controller from gaming
- _(theming)_ Make checkboxes easier to see, misc fixes
- _(maintenance)_ Make it clear how to merge pacdiff, drop kompare, warn about destructiveness
- _(kernels)_ Update kernel state after kernels initialized for the first time
- _(maintenance)_ Pacdiff file not being deleted after y
- _(kernels)_ Distinguish broken and non-installed modules
- _(configService)_ Too many update executions
- _(http)_ Downgrade tauri http plugin, adapter currently breaks actions
- _(kernels)_ Remove unnecessary double call
- _(translations)_ Missing translations with angular optimization
- Focus active window when opening rani a second time

### 🚜 Refactor

- Move status data to cached service, allow running external AUR update

### 🎨 Styling

- _(prettier)_ Set tailwindcss plugin active and reformat

### ⚙️ Miscellaneous Tasks

- Post nx-migration cleanup invalid tests
- Set nx defaults for generation of components

## [2.3.0] - 2025-03-21

### 🚀 Features

- _(packages)_ Use searchable table instead for display
- _(system-tools)_ Only load components when tab has been selected
- _(kernels)_ Add kernels component
- _(kernels)_ Add dkms module check and loading indicator
- _(gaming)_ Add more WINE versions
- _(language-packs)_ New component
- _(settings)_ Move to dedicated page
- _(maintenance)_ Allow merging pacdiff files interactively
- _(system-status)_ Warn about pending reboot
- _(system-settings)_ Add bpftune

### 🐛 Bug Fixes

- _(system-components)_ Scanning-support never being enabled
- _(os-interact)_ Check for systemd sockets as well, fixing non-recognized sockets
- _(pnpm-lock.yaml)_ Sync with package.json
- _(config-service)_ Apply Loglevel on settings change
- _(notifications)_ Unbreak not-sending notifications

### 🚜 Refactor

- Massively improve performance by caching and using services
- Use pacman regex for prefiltering, print commands used

### 📚 Documentation

- _(changelog)_ Update

### ⚙️ Miscellaneous Tasks

- Drop ununsed code
- Fix commitizen installation

## [2.2.0] - 2025-03-18

### 🚀 Features

- _(system-settings)_ Add packages section
- _(translation)_ Use inbuilt Tauri resources to load translations
- Version 2.2.0

### 🐛 Bug Fixes

- _(system-components)_ Virtualbox checking for group rather than pkg
- _(packages)_ Invalid pkgnames, fixed parser as well
- _(language)_ Only update language once on startup
- _(translations)_ Safeguard for failing to load translation files

## [2.1.0] - 2025-03-18

### 🚀 Features

- _(garuda-update)_ Set GARUDA_UPDATE_RANI=1 to notify garuda-update of usage via rani

### 🐛 Bug Fixes

- _(vmware,virtualbox)_ Disable DMABUF renderer
- _(lib.rs)_ Attempt fixing detection by making strings lowercase
- _(dynamic-checkboxes)_ Allow disabled entries to be properly managed.

### 📚 Documentation

- _(changelog)_ Add changelog

### ⚙️ Miscellaneous Tasks

- _(flake.lock)_ Update

## [2.0.0] - 2025-03-17

### 🚀 Features

- _(window)_ Responsive window button states
- _(loadingservice)_ Use refcount on loading indicator
- _(font)_ Use system-ui
- _(init)_ Resize window to reasonable size when monitor is too small
- Move parseArchWiki script to Typescript
- _(system-status)_ Report AUR updates too
- _(system-status)_ Support schedule AUR updates, update on task updates
- Strip AUR functionality, we decided to not support it
- _(system-components)_ Expand with containers section
- Menu bar "terminal" button glows actively when tasks are pending
- _(terminal)_ Only show buttons when needed/reasonable

### 🐛 Bug Fixes

- _(os-interact)_ Fix setting dns to default
- _(diagnostics)_ Disable logging symbol after loading full diagnostics
- _(system-status)_ Invalid update versions
- _(angular)_ Zoneless change detection complaining
- _(gamer)_ Some icons being improperly sized
- _(gaming)_ Missing pacakges, add script to check for missing ones
- _(theme)_ Light theme not working, wrong window buttons on the right
- _(window)_ Set reasonable menu breakpoint, set logicalSize dynamically
- _(dynamic-checkboxes)_ Ensure already enabled/installed packages are not disabled
- Duplicated updates, duplicated logic
- _(systemd-services)_ Event reporting failure due to no output
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

- _(package.json,cargo.toml)_ Cleanup required packages
- _(translation)_ Load translations before the application loads
- _(home)_ Utilize launch-terminal garuda-libs script for chroot

## [1.2.0] - 2025-03-05

### 🚀 Features

- Allow confirmation-less app exit when there are no pending tasks.
- _(operation-manager)_ Do not store/restore pending operations when exiting app
- _(logs)_ Allow setting loglevel

### 🐛 Bug Fixes

- _(shell.nix)_ Fix nix build env
- _(style)_ Disable any kind of text selection
- _(diagnostics)_ Do not use garuda-inxi funstuff
- _(config,home)_ Fix excessive redraws, live system detection, angular change detection
- More efficient file existence checks via plugin-fs
- _(status)_ Logs reporting update check failure when none are availble
- _(theme-handler)_ Use updateConfig method to set new setting
- _(xterm)_ Use computed signal to prevent wrong theme

### 🚜 Refactor

- _(first-boot)_ Revamp first boot checks
- _(darkmode)_ Redo darkmode to be more consistent
- _(language-switcher)_ Redo language-switcher to be more consistent
- _(app)_ Pre-init config before rendering application
- _(app)_ More reliable menu/label setup, more reliable transloco

## [1.1.0] - 2025-03-02

### 🚀 Features

- Run setup-assistant on first boot, relaunch after update
- _(gaming)_ Import new icons from gamer, placeholders update

## [1.0.2] - 2025-03-02

### 🚀 Features

- _(welcome)_ Don't show status on live system

### 🐛 Bug Fixes

- No max size, inform about errors, calamares not starting

## [1.0.1] - 2025-03-02

### 🐛 Bug Fixes

- User set to null, loglevel to info, skip mirrorlist by default

## [1.0.0] - 2025-03-02

### 🚀 Features

- Safeguards app shutdown/remove action, only safe undone
- _(operations)_ Allow aborting running, and running directly
- _(system-status)_ Warn on >2w no update
- Add autostart config, remove problematic vboxkvm

### 🐛 Bug Fixes

- _(nvidia)_ Attempt disabling dmabuf renderer
- _(gaming)_ Invalid parsed entries
- _(operation-manager)_ Passed signal instead of value
- _(diagnostics)_ Make buttons not rounded, comply with rest of app

### ⚙️ Miscellaneous Tasks

- Extend Arch wiki list parsing

## [0.2.0] - 2025-03-01

### 🚀 Features

- _(privilege-manager)_ Keep one-time use creds between a run of multiple
- Make settings menu entries dynamic, use fontawesome icons
- Improve visuals on home and other places
- Set reasonable min window size, report progress, misc fixes
- _(gaming)_ Derive list of games from AUR, check whether available
- _(operations)_ Sort pending list after order, cleanup subscriptions
- Use cdr onpush for better performance, support installing AUR games
- Support reporting pacdiff/update status on home, allow ensuring package is installed

### 🐛 Bug Fixes

- Save state on shutdown
- _(system-settings)_ Toggling hblock now works as expected
- Singleton ConfigService, shell not initializing
- _(privilege-manager)_ Handle aborted input gracefully
- _(checkboxes)_ Fix unexpected toggle behaviour
- _(diagnostics)_ Page not loading due to privatebinclient constructor missing
- _(deps)_ Add missing dep, remove obsolete
- _(window)_ Add missing permission
- Hblock enable action, don't try translating game titles
- Use better icons for status

### 🚜 Refactor

- _(configService)_ Move user determination to configService
- _(configService)_ Source darkMode from configService

### 📚 Documentation

- Add deps, run funstuff

### ⚙️ Miscellaneous Tasks

- Add linting
- _(types)_ Declare a handful interfaces as types
- Version 0.2.0

## [0.1.1] - 2025-02-27

### 🚀 Features

- _(terminal)_ Use WebGlAddon

### 🐛 Bug Fixes

- _(gamer)_ No more broken icons
- _(privilege-manager)_ Don't dump password into logs

## [0.1.0] - 2025-02-27

### 🚀 Features

- _(logger)_ Introduce custom logger class for loglevels
- _(translations)_ Add trashy auto-translated languages
- _(shortcuts)_ Prevent default browser shortcuts in prod

### 🐛 Bug Fixes

- _(translations)_ Disable script optimisations in prod mode

### ⚙️ Miscellaneous Tasks

- Version 0.1.0

## [0.0.1] - 2025-02-27

### 🚀 Features

- Initial commit
- _(gamer)_ Finished queue logic, much more entries
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

- _(privilege-manager)_ Wrong authenticated status
- Operation management, show disabled services
- Direct run should open terminal
- Await store being ready, make initial window higher, readme link

### 🚜 Refactor

- Migrate to configService

### 📚 Documentation

- _(readme)_ Add

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
