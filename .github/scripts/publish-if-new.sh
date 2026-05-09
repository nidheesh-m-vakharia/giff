#!/usr/bin/env bash
# Publishes a crate to crates.io iff its local Cargo.toml version differs from the
# latest version on crates.io. No-op if they match. Used by .github/workflows/release.yml.
#
# Usage:
#   ./.github/scripts/publish-if-new.sh <crate-name>
#
# Required env:
#   CARGO_REGISTRY_TOKEN — populated from the repository secret.
#
# Side effects (read by the calling workflow when crate == "giffstack"):
#   GIFFSTACK_PUBLISHED=true|false
#   GIFFSTACK_VERSION=<version>
#
# These are written to $GITHUB_ENV so subsequent workflow steps can decide whether to
# tag and create a GitHub release.

set -euo pipefail

CRATE="${1:?usage: publish-if-new.sh <crate-name>}"

# Local version. cargo metadata is the canonical source.
LOCAL_VERSION=$(cargo metadata --no-deps --format-version=1 \
  | python3 -c "import sys, json; m=json.load(sys.stdin); print(next(p['version'] for p in m['packages'] if p['name']=='${CRATE}'))")

# Latest published version. The crates.io API returns 404 if the crate has never been
# published — treat that as "definitely needs publishing."
PUBLISHED_VERSION=$(
  curl -fsS "https://crates.io/api/v1/crates/${CRATE}" 2>/dev/null \
    | python3 -c "import sys, json; print(json.load(sys.stdin)['crate']['max_version'])" 2>/dev/null \
    || echo ""
)

echo "  local:     ${LOCAL_VERSION}"
echo "  published: ${PUBLISHED_VERSION:-<none>}"

PUBLISHED_THIS_RUN=false
if [[ "${LOCAL_VERSION}" == "${PUBLISHED_VERSION}" ]]; then
  echo "  → already on crates.io; skipping."
else
  echo "  → publishing ${CRATE} ${LOCAL_VERSION} ..."
  cargo publish -p "${CRATE}" --token "${CARGO_REGISTRY_TOKEN}"
  PUBLISHED_THIS_RUN=true
fi

# Surface the giffstack-specific outcome to the workflow so it can decide whether to
# create a tag + GitHub release for this push.
if [[ "${CRATE}" == "giffstack" ]]; then
  {
    echo "GIFFSTACK_PUBLISHED=${PUBLISHED_THIS_RUN}"
    echo "GIFFSTACK_VERSION=${LOCAL_VERSION}"
  } >> "${GITHUB_ENV:-/dev/null}"
fi
