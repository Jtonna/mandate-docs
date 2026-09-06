#!/usr/bin/env bash
# Lifecycle commands for the quarantine containers created by run.sh.
#
#   ./harness.sh list
#   ./harness.sh shell <identifier>
#   ./harness.sh rm <identifier>
#   ./harness.sh rm-all

set -euo pipefail

PREFIX="mandate-docs-test-codebase"
LABEL="org.mandate-docs.harness=test-codebase"

usage() {
  cat >&2 <<'USAGE'
Usage: harness.sh <command> [identifier]

  list              List the harness containers and their state.
  shell <id>        Open a shell in a container, starting it if it is stopped.
  rm <id>           Stop and remove a container and its image.
  rm-all            Stop and remove every harness container and image.
USAGE
  exit 2
}

command -v docker >/dev/null 2>&1 || { echo "docker is not on PATH." >&2; exit 1; }

[ $# -ge 1 ] || usage
CMD="$1"; shift

name_for() {
  [ $# -ge 1 ] && [ -n "$1" ] || { echo "This command needs an identifier." >&2; usage; }
  printf '%s-%s' "$PREFIX" "$1"
}

require_container() {
  if ! docker ps -a --format '{{.Names}}' | grep -Fxq "$1"; then
    echo "No container named $1. Run ./harness.sh list to see what exists." >&2
    exit 1
  fi
}

case "$CMD" in
  list)
    docker ps -a --filter "label=$LABEL" \
      --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}'
    ;;

  shell)
    NAME="$(name_for "${1:-}")"
    require_container "$NAME"
    if [ "$(docker inspect -f '{{.State.Running}}' "$NAME")" != "true" ]; then
      docker start "$NAME" >/dev/null
    fi
    exec docker exec -it -w /repo "$NAME" sh
    ;;

  rm)
    NAME="$(name_for "${1:-}")"
    require_container "$NAME"
    docker rm -f "$NAME" >/dev/null
    docker image rm -f "$NAME" >/dev/null 2>&1 || true
    echo "Removed $NAME"
    ;;

  rm-all)
    NAMES="$(docker ps -a --filter "label=$LABEL" --format '{{.Names}}')"
    if [ -z "$NAMES" ]; then
      echo "No harness containers to remove."
      exit 0
    fi
    echo "$NAMES" | while read -r n; do
      [ -n "$n" ] || continue
      docker rm -f "$n" >/dev/null
      docker image rm -f "$n" >/dev/null 2>&1 || true
      echo "Removed $n"
    done
    ;;

  -h|--help|help) usage ;;
  *) echo "Unknown command: $CMD" >&2; usage ;;
esac
