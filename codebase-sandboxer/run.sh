#!/usr/bin/env bash
# Clone a repository into an isolated container.
#
#   ./run.sh <repo-url> [identifier]
#
# The identifier defaults to the owner and repository from the URL. The volume
# and the container are both named mandate-ext-test-repo-<identifier>, and the
# clone lives at /repo.
#
# One shared image holds git and a shell. The repository lives in a named
# volume, mounted read-only into a container that has no network.

set -euo pipefail

# Git Bash on Windows rewrites arguments that look like unix paths into Windows
# paths, which turns /repo into C:/Program Files/Git/repo before docker sees it.
# Unset elsewhere this variable does nothing.
export MSYS_NO_PATHCONV=1
export MSYS2_ARG_CONV_EXCL="*"

PREFIX="mandate-ext-test-repo"
IMAGE="mandate-ext-test-repo-base"

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
  echo "Remove it with: docker rm -f $NAME && docker volume rm $NAME" >&2
  exit 1
fi

if docker volume ls --format '{{.Name}}' | grep -Fxq "$NAME"; then
  echo "A volume named $NAME already exists." >&2
  echo "Remove it with: docker volume rm $NAME" >&2
  exit 1
fi

HERE="$(cd "$(dirname "$0")" && pwd)"

# The shared image is built once and reused. Rebuild it by hand when the
# Dockerfile changes: docker build -t mandate-ext-test-repo-base codebase-sandboxer
if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "Building shared image $IMAGE"
  docker build -q -t "$IMAGE" -f "$HERE/Dockerfile" "$HERE" >/dev/null
fi

docker volume create "$NAME" >/dev/null

# Phase 1. A throwaway container with network access clones into the volume.
# core.hooksPath=/dev/null stops a repository-supplied hook from running.
# protocol.file.allow=never blocks a submodule pointing at a local path.
echo "Cloning $REPO_URL"
if ! docker run --rm \
  --mount "type=volume,source=$NAME,target=/repo" \
  --cap-drop ALL \
  --security-opt no-new-privileges \
  "$IMAGE" \
  git -c core.hooksPath=/dev/null -c protocol.file.allow=never \
      clone --no-recurse-submodules "$REPO_URL" /repo
then
  echo "Clone failed. Removing the volume." >&2
  docker volume rm "$NAME" >/dev/null 2>&1 || true
  exit 1
fi

# Phase 2. The long-lived container. No network, and the repository is mounted
# read-only so nothing inside can alter the code it is examining.
docker create \
  --name "$NAME" \
  --mount "type=volume,source=$NAME,target=/repo,readonly" \
  --network none \
  --cap-drop ALL \
  --security-opt no-new-privileges \
  "$IMAGE" >/dev/null

cat <<EOM

Container: $NAME (stopped)
Volume:    $NAME
Clone:     /repo (read-only)

  docker start $NAME && docker exec -it $NAME sh
  docker rm -f $NAME && docker volume rm $NAME
EOM
