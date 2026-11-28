# shellcheck shell=bash
# The Lima files the self-test scripts copy into the machine under test.
# Paths are relative to the repository root. Sourced, never run.
# shellcheck disable=SC2034 # read by the scripts that source this file
pacman_conf_fixture=crates/system/src/packages/pacman_conf/fixtures/pacman.conf
# shellcheck disable=SC2034 # read by the scripts that source this file
locale_gen_fixture=crates/plan/src/testing/locales/fixtures/locale.gen
# shellcheck disable=SC2034 # read by the scripts that source this file
merge_fixtures=("$pacman_conf_fixture" "$locale_gen_fixture")
