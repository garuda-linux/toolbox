#!/usr/bin/env bash
#   scripts/coverage.sh [output dir]   (default: target/coverage/report)
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
out=$(realpath -m "${1:-$repo/target/coverage/report}")
suites=${COVERAGE_SELFTESTS-all}
floor=${COVERAGE_FAIL_UNDER:-}
# Tests and code no runner can reach: dev CLIs, the real PrivateBin transport, the Wayland raise.
ignore=(--ignore-filename-regex '(^|/)tests(\.rs$|/)|app/(xtask|lint)/src/main\.rs$|crates/privatebin/src/https\.rs$|crates/instance/src/raise(\.rs$|/)')

cd "$repo"
export CARGO_TARGET_DIR="$repo/target/coverage"
# shellcheck source=/dev/null
source <(cargo llvm-cov show-env --sh)
cargo llvm-cov clean --workspace

failed=()
cargo nextest run --workspace --features toolbox-gui/testing --profile ci || failed+=("tests")

for suite in $suites; do
  "$repo/scripts/selftest.sh" "$suite" || failed+=("selftest $suite")
done

mkdir -p "$out"
cargo llvm-cov report "${ignore[@]}" --cobertura --output-path "$out/cobertura.xml"
cargo llvm-cov report "${ignore[@]}" --html --output-dir "$out"
summary=$(cargo llvm-cov report "${ignore[@]}" --summary-only)
printf '%s\n' "$summary"

# llvm-cov's TOTAL row, because Cobertura's line-rate counts lines differently.
awk '$1 == "TOTAL" { print "Line coverage: " $10 }' <<<"$summary"
if [ ${#failed[@]} -gt 0 ]; then
  echo "Failed while measuring (the report covers what ran): ${failed[*]}" >&2
fi
echo "Report: $out/cobertura.xml, $out/html/index.html"

if [ -n "$floor" ]; then
  cargo llvm-cov report "${ignore[@]}" --fail-under-lines "$floor" --summary-only >/dev/null
fi
