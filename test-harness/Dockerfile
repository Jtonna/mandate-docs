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

ARG REPO_URL

# GIT_TERMINAL_PROMPT=0 makes a private or misspelled URL fail instead of
# blocking the build on a credential prompt.
ENV GIT_TERMINAL_PROMPT=0

# core.hooksPath=/dev/null stops a repository-supplied hook from running.
# protocol.file.allow=never blocks a submodule pointing at a local path.
RUN set -eu; \
    if [ -z "${REPO_URL}" ]; then echo "REPO_URL build argument is required" >&2; exit 1; fi; \
    git -c core.hooksPath=/dev/null -c protocol.file.allow=never \
        clone --no-recurse-submodules "${REPO_URL}" /repo

WORKDIR /repo

# The container exists to be shelled into. It runs no repository code and holds
# no service open.
CMD ["tail", "-f", "/dev/null"]
