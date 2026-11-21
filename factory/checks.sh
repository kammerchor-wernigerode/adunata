#!/usr/bin/env bash
# Adunata's checks up to integration. Exit 0 only when the branch is fit to review.
#
# The factory's gate runs this script from the main checkout with a job's worktree as the
# working directory. People run it from the root of their checkout before a push.
# With Nix it builds the flake target `integration`, which sees only files that Git tracks.
# Without Nix it runs the same checks one by one and needs the services of compose.yaml.
set -euo pipefail

if command -v nix >/dev/null 2>&1; then
  system=$(nix eval --impure --raw --expr builtins.currentSystem)
  exec nix build --print-build-logs --no-link ".#checks.$system.integration"
fi

(
  cd client
  pnpm install --frozen-lockfile
  pnpm run lint
  pnpm run lint:editorconfig
  pnpm run lint:licenses
)
(
  cd server
  cargo fmt --check
  cargo clippy --all-targets -- --deny warnings
  cargo deny check bans licenses sources
)
(cd client && pnpm run build)
(
  cd server
  cargo about generate --frozen --fail about.hbs >> ../client/dist/licenses.txt
  cargo build
  cargo test --bins -- --skip integration::
)
(cd client && pnpm test)
(
  cd server
  cargo test --bins -- integration::
  cargo test --test startup
)
(
  cd client
  pnpm run test:integration
  pnpm run check:generated
)
node --test .github/scripts/*.test.mjs
