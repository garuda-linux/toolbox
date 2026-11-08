#!/usr/bin/env bash
#   scripts/selftest-junit.sh <selftest root> <output.xml>
set -euo pipefail

root=$1
output=$2

escape() {
  sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' -e 's/"/\&quot;/g'
}

# XML forbids the CDATA end marker and control characters but tab and newline.
cdata() {
  tr -d '\000-\010\013\014\016-\037' | sed -e 's/]]>/]]]]><![CDATA[>/g'
}

cases=0 failures=0 total=0
body=""
for run in "$root"/*/; do
  run=${run%/}
  [ -f "$run/result" ] || continue
  name=$(basename "$run")
  read -r code elapsed <"$run/result"
  cases=$((cases + 1))
  total=$((total + elapsed))
  body+="  <testcase classname=\"selftest\" name=\"$(printf '%s' "$name" | escape)\" time=\"$elapsed\">"$'\n'
  if [ "$code" != 0 ]; then
    failures=$((failures + 1))
    message=$(grep -m1 "SELFTEST FAIL" "$run/out.log" 2>/dev/null || true)
    if [ -z "$message" ]; then
      if [ "$code" = 124 ]; then
        message="timed out"
      else
        message="exit $code without a SELFTEST result"
      fi
    fi
    body+="    <failure message=\"$(printf '%s' "$message" | escape)\"><![CDATA["
    body+=$(tail -n 200 "$run/out.log" 2>/dev/null | cdata || true)
    body+="]]></failure>"$'\n'
  fi
  body+="  </testcase>"$'\n'
done

{
  printf '<?xml version="1.0" encoding="UTF-8"?>\n'
  printf '<testsuites>\n<testsuite name="selftest" tests="%s" failures="%s" time="%s">\n' "$cases" "$failures" "$total"
  printf '%s' "$body"
  printf '</testsuite>\n</testsuites>\n'
} >"$output"
