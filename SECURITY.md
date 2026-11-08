# Security policy

## Reporting a vulnerability

Do not report security problems in a public issue. Open a confidential issue on
[gitlab.com/garuda-linux/applications/toolbox](https://gitlab.com/garuda-linux/applications/toolbox/-/issues/new) and
tick "This issue is confidential". Include the version, the system (installed or live ISO) and the steps to reproduce.

## Scope

The app runs as the user. Only the helper `garuda-toolbox-helper` runs as root, started with `pkexec` through the polkit
actions `org.garudalinux.toolbox.*`. Problems in the helper, the polkit actions or the handling of text from packages,
config files and program output matter most. [docs/agents/security.md](docs/agents/security.md) lists what runs with
which rights.

## Supported versions

Only the latest release gets security fixes.
