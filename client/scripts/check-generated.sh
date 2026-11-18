#!/bin/sh
# Regenerates the client code from openapi.json and fails if it differs from the code found. Restores the code found on every exit.
set -eu
cd "$(dirname "$0")/.."

generated=src/api/generated
found=$(mktemp -d)
cp -R "$generated" "$found/"

restore() {
  rm -rf "$generated"
  cp -R "$found/generated" "$generated"
  rm -rf "$found"
}
trap restore EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

pnpm exec orval
if ! diff -ru "$found/generated" "$generated"; then
  echo "Generated client code is out of date. Run: pnpm run generate" >&2
  exit 1
fi
