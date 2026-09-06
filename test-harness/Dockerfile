# Quarantine container for a third-party repository.
#
# The clone happens at build time and nothing from the cloned repository is ever
# executed, here or at run time. No package manager reads the repo's manifests,
# no install script runs, no build step runs. The image contains git, a shell and
# the working copy, and that is all.

FROM alpine:3.20

RUN apk add --no-cache git ca-certificates \
 && adduser -D -u 10001 -h /home/quarantine quarantine \
 && mkdir -p /repo \
 && chown quarantine:quarantine /repo

USER quarantine
WORKDIR /home/quarantine

# Repository to clone. REPO_URL is required. REPO_REF is a branch or tag name,
# empty for the remote's default branch. CLONE_DEPTH is a shallow clone depth,
# empty for a full clone.
ARG REPO_URL
ARG REPO_REF=""
ARG CLONE_DEPTH=""

# GIT_TERMINAL_PROMPT=0 makes a private or misspelled URL fail instead of
# blocking the build on a credential prompt.
ENV GIT_TERMINAL_PROMPT=0

RUN set -eu; \
    if [ -z "${REPO_URL}" ]; then echo "REPO_URL build argument is required" >&2; exit 1; fi; \
    set -- clone --no-recurse-submodules; \
    if [ -n "${REPO_REF}" ]; then set -- "$@" --branch "${REPO_REF}"; fi; \
    if [ -n "${CLONE_DEPTH}" ]; then set -- "$@" --depth "${CLONE_DEPTH}"; fi; \
    git -c core.hooksPath=/dev/null -c protocol.file.allow=never "$@" "${REPO_URL}" /repo

LABEL org.mandate-docs.harness="test-codebase"
LABEL org.mandate-docs.repo-url="${REPO_URL}"

WORKDIR /repo

# The container exists to be shelled into. It runs no repository code and holds
# no service open.
CMD ["tail", "-f", "/dev/null"]
