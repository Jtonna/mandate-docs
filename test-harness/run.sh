#!/usr/bin/env bash
# Clone a repository into an isolated container.
#
#   ./run.sh <repo-url> [identifier]
#
# The identifier defaults to the owner and repository from the URL. The
# container is named
# mandate-ext-repo-test-<identifier> and the clone lives at /repo.

set -euo pipefail

PREFIX="mandate-ext-repo-test"

if [ $# -lt 1 ] || [ $# -gt 2 ] || [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  echo "Usage: run.sh <repo-url> [identifier]" >&2
  exit 2
fi

REPO_URL="$1"
IDENTIFIER="${2:-}"

command -v docker >/dev/null 2>&1 || { echo "docker is not on PATH." >&2; exit 1; }

# Derive the identifier from the owner and repository in the URL when not given.
# Docker names cannot contain a slash or colon, so the scheme and host are
# dropped and the remaining path segments are joined with dashes.
if [ -z "$IDENTIFIER" ]; then
  IDENTIFIER="${REPO_URL%/}"
  IDENTIFIER="${IDENTIFIER%.git}"
  IDENTIFIER="${IDENTIFIER#*://}"          # drop scheme
  IDENTIFIER="${IDENTIFIER#*@}"            # drop any user@ in an ssh url
  IDENTIFIER="${IDENTIFIER#*[/:]}"         # drop host
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

# create, not run. The clone already happened at build time, so the container
# has nothing to do until you want a shell in it. It is left stopped.
docker create \
  --name "$NAME" \
  --network none \
  --cap-drop ALL \
  --security-opt no-new-privileges \
  "$NAME" >/dev/null

cat <<EOM

Container: $NAME (stopped)
Clone:     /repo

  docker start $NAME && docker exec -it $NAME sh
  docker rm -f $NAME
EOM
