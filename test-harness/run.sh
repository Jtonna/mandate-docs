#!/usr/bin/env bash
# Clone a repository into an isolated container.
#
#   ./run.sh <repo-url> [identifier]
#
# The identifier defaults to the repository name. The container is named
# mandate-docs-test-codebase-<identifier> and the clone lives at /repo.

set -euo pipefail

PREFIX="mandate-docs-test-codebase"

if [ $# -lt 1 ] || [ $# -gt 2 ] || [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  echo "Usage: run.sh <repo-url> [identifier]" >&2
  exit 2
fi

REPO_URL="$1"
IDENTIFIER="${2:-}"

command -v docker >/dev/null 2>&1 || { echo "docker is not on PATH." >&2; exit 1; }

# Derive the identifier from the last path segment of the URL when not given,
# then reduce it to lowercase characters Docker accepts in a name.
if [ -z "$IDENTIFIER" ]; then
  IDENTIFIER="${REPO_URL%/}"
  IDENTIFIER="${IDENTIFIER##*/}"
  IDENTIFIER="${IDENTIFIER%.git}"
fi
IDENTIFIER="$(printf '%s' "$IDENTIFIER" \
  | tr '[:upper:]' '[:lower:]' \
  | sed -e 's/[^a-z0-9._-]/-/g' -e 's/^[._-]*//' -e 's/[._-]*$//')"

if [ -z "$IDENTIFIER" ]; then
  echo "Could not derive an identifier from '$REPO_URL'. Pass one explicitly." >&2
  exit 2
fi

NAME="${PREFIX}-${IDENTIFIER}"

if docker ps -a --format '{{.Names}}' | grep -Fxq "$NAME"; then
  echo "A container named $NAME already exists." >&2
  echo "Remove it with: docker rm -f $NAME" >&2
  exit 1
fi

HERE="$(cd "$(dirname "$0")" && pwd)"

docker build \
  --build-arg "REPO_URL=$REPO_URL" \
  -t "$NAME" \
  -f "$HERE/Dockerfile" \
  "$HERE"

docker run -d \
  --name "$NAME" \
  --network none \
  --cap-drop ALL \
  --security-opt no-new-privileges \
  "$NAME" >/dev/null

cat <<EOM

Container: $NAME
Clone:     /repo

  docker exec -it $NAME sh
  docker rm -f $NAME
EOM
