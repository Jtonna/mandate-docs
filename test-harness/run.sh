#!/usr/bin/env bash
# Clone a third-party repository into an isolated container.
#
#   ./run.sh <repo-url> [identifier] [--ref <branch-or-tag>] [--depth <n>]
#
# The identifier defaults to the repository name. The container is named
# mandate-docs-test-codebase-<identifier> and the clone lives at /repo.

set -euo pipefail

PREFIX="mandate-docs-test-codebase"

usage() {
  cat >&2 <<'USAGE'
Usage: run.sh <repo-url> [identifier] [--ref <branch-or-tag>] [--depth <n>]

  <repo-url>    Repository to clone, for example https://github.com/owner/name
  [identifier]  Name suffix for the container. Defaults to the repository name.
  --ref         Branch or tag to check out. Defaults to the remote default branch.
  --depth       Shallow clone depth. Defaults to a full clone.
USAGE
  exit 2
}

REPO_URL=""
IDENTIFIER=""
REF=""
DEPTH=""

while [ $# -gt 0 ]; do
  case "$1" in
    --ref)   [ $# -ge 2 ] || usage; REF="$2"; shift 2 ;;
    --depth) [ $# -ge 2 ] || usage; DEPTH="$2"; shift 2 ;;
    -h|--help) usage ;;
    -*) echo "Unknown option: $1" >&2; usage ;;
    *)
      if [ -z "$REPO_URL" ]; then REPO_URL="$1"
      elif [ -z "$IDENTIFIER" ]; then IDENTIFIER="$1"
      else echo "Unexpected argument: $1" >&2; usage
      fi
      shift ;;
  esac
done

[ -n "$REPO_URL" ] || usage

if [ -n "$DEPTH" ] && ! printf '%s' "$DEPTH" | grep -Eq '^[1-9][0-9]*$'; then
  echo "--depth must be a positive integer, got: $DEPTH" >&2
  exit 2
fi

command -v docker >/dev/null 2>&1 || { echo "docker is not on PATH." >&2; exit 1; }

# Derive the identifier from the last path segment of the URL when not given,
# then reduce whichever we have to lowercase alphanumerics, dashes and dots.
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
  cat >&2 <<EOM
A container named $NAME already exists.

  Re-enter it:  ./harness.sh shell $IDENTIFIER
  Remove it:    ./harness.sh rm $IDENTIFIER

Or pass a different identifier as the second argument.
EOM
  exit 1
fi

HERE="$(cd "$(dirname "$0")" && pwd)"

echo "Building image $NAME from $REPO_URL"
docker build \
  --pull \
  --build-arg "REPO_URL=$REPO_URL" \
  --build-arg "REPO_REF=$REF" \
  --build-arg "CLONE_DEPTH=$DEPTH" \
  --label "org.mandate-docs.harness=test-codebase" \
  -t "$NAME" \
  -f "$HERE/Dockerfile" \
  "$HERE"

echo "Starting container $NAME"
docker run -d \
  --name "$NAME" \
  --label "org.mandate-docs.harness=test-codebase" \
  --network none \
  --cap-drop ALL \
  --security-opt no-new-privileges \
  --pids-limit 512 \
  --memory 2g \
  "$NAME" >/dev/null

cat <<EOM

Container: $NAME
Clone:     /repo (inside the container)
Network:   disabled
Mounts:    none

Shell in:  ./harness.sh shell $IDENTIFIER
           docker exec -it $NAME sh
Remove:    ./harness.sh rm $IDENTIFIER
EOM
