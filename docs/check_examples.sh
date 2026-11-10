#!/bin/sh
# Run every `pk` command in the docs that uses the bundled sample data, from the rust
# folder, and fail if any of them errors. Needs target/release/pk.
set -eu
cd "$(dirname "$0")/.."
PK=./target/release/pk
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
fails=0
total=0
# Commands, one per line, in the order they appear. Runs in a scratch copy so --out files
# don't land in the source tree, and so steps that read an earlier step's output work.
cp -R examples "$TMP/examples"
grep -h -E '^\s*pk (describe|fit|li600|concat)' docs/docs/*.md docs/docs/*/*.md | sed -E 's/^[[:space:]]+//; s/[[:space:]]+#.*$//' > "$TMP/commands.txt"
while IFS= read -r line; do
  case "$line" in
    *"<"*|*"..."*) continue ;;  # syntax lines with placeholders, not examples
  esac
  case "$line" in
    *examples/*|"pk describe"*|*li600_corrected.csv*|*fvcb_fit.json*) ;;
    *) continue ;;
  esac
  total=$((total + 1))
  if (cd "$TMP" && eval "\"$OLDPWD/$PK\" ${line#pk }" > out.txt 2>&1); then
    :
  else
    echo "FAILED: $line"; head -c 400 "$TMP/out.txt"; echo
    fails=$((fails + 1))
  fi
done < "$TMP/commands.txt"
# Every error code pk knows is in the table in reference/commands.md, and every warning
# code is in the table in reference/output.md.
"$PK" describe > "$TMP/describe.json"
codes() { python3 -c "import json,sys; print('\n'.join(c['code'] for c in json.load(open(sys.argv[1]))[sys.argv[2]]['codes']))" "$TMP/describe.json" "$1"; }
for code in $(codes errors); do
  if ! grep -q "| \`$code\` |" docs/docs/reference/commands.md; then
    echo "Error code $code is missing from docs/docs/reference/commands.md"
    fails=$((fails + 1))
  fi
done
for code in $(codes warnings); do
  if ! grep -q "| \`$code\` |" docs/docs/reference/output.md; then
    echo "Warning code $code is missing from docs/docs/reference/output.md"
    fails=$((fails + 1))
  fi
done
echo "$total doc commands run, $fails failed"
[ "$fails" -eq 0 ]
