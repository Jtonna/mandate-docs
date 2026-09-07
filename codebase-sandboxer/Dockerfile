# The shared harness image. One image serves every repository.
#
# It holds git, a shell and nothing else. No repository is baked in: a clone
# lives in a named volume mounted at /repo, so the image stays an environment
# rather than data.
#
# Built once as mandate-ext-test-repo-base:latest by run.sh.

FROM alpine:3.20

RUN apk add --no-cache git ca-certificates \
 && adduser -D -u 10001 -h /home/quarantine quarantine \
 && mkdir -p /repo \
 && chown quarantine:quarantine /repo

# A fresh named volume mounted at /repo inherits this ownership, which is what
# lets the clone run as a non-root user.
USER quarantine
WORKDIR /repo

# GIT_TERMINAL_PROMPT=0 makes a private or misspelled URL fail instead of
# blocking on a credential prompt.
ENV GIT_TERMINAL_PROMPT=0

# Inspection containers run this and nothing else. The image never executes
# anything from a cloned repository.
CMD ["tail", "-f", "/dev/null"]
