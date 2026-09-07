# test-harness

One script. Give it a repository URL, get a container with that repository
cloned inside it.

```bash
./run.sh https://github.com/spring-projects/spring-petclinic
```

Bash only. Run it from Git Bash on Windows.

Output:

```
Container: mandate-ext-test-repo-spring-projects-spring-petclinic (stopped)
Volume:    mandate-ext-test-repo-spring-projects-spring-petclinic
Clone:     /repo (read-only)

  docker start <name> && docker exec -it <name> sh
  docker rm -f <name> && docker volume rm <name>
```

## Why

Third-party repositories are untrusted code. Cloning them onto the host puts
their hooks, submodules, and build scripts one careless command away from
running. A container keeps each one in its own box, and the predictable name
makes it obvious which is which.

One repository per container. No shared state between them.

## How it is put together

One image is the environment. Each repository is a volume. The container joins
them.

```
mandate-ext-test-repo-base        one image, git and a shell, built once

per repository:
  volume     mandate-ext-test-repo-<identifier>    the clone
  container  mandate-ext-test-repo-<identifier>    mounts it read-only
```

Cloning happens in a throwaway container that has network access and is deleted
the moment it finishes. The container you keep has no network at all, because a
container's network mode is fixed when it is created and cannot be dropped
later.

The repository never enters the image. That keeps the image an environment
rather than data, so twenty repositories still means one image.

## Naming

```
mandate-ext-test-repo-<identifier>
```

The identifier defaults to the owner and repository from the URL. Docker names
cannot contain a slash or a colon, so the scheme and host are dropped and what
remains is joined with dashes:

```
https://github.com/spring-projects/spring-petclinic.git
  -> mandate-ext-test-repo-spring-projects-spring-petclinic

git@github.com:owner/name.git
  -> mandate-ext-test-repo-owner-name
```

Including the owner means two repositories with the same name from different
owners do not collide.

Pass a second argument to override it:

```bash
./run.sh https://github.com/owner/name my-label
# mandate-ext-test-repo-my-label
```

Either way it is lowercased, and anything outside `a-z0-9._-` becomes a dash.
The container and the volume take the same name. If either already exists the
script stops rather than replacing it.

## Managing containers

There is no wrapper. These are plain Docker commands:

```bash
docker ps -a --filter "name=mandate-ext-test-repo-"   # list them
docker volume ls --filter "name=mandate-ext-test-repo-"
docker start mandate-ext-test-repo-<id>                # start one
docker exec -it mandate-ext-test-repo-<id> sh          # shell in
docker stop mandate-ext-test-repo-<id>                 # stop it again
```

Removing a repository takes two commands, because the container and its volume
are separate objects:

```bash
docker rm -f mandate-ext-test-repo-<id>
docker volume rm mandate-ext-test-repo-<id>
```

Remove everything the harness made:

```bash
docker rm -f $(docker ps -aq --filter "name=mandate-ext-test-repo-")
docker volume rm $(docker volume ls -q --filter "name=mandate-ext-test-repo-")
docker rmi mandate-ext-test-repo-base
```

The shared image is built on first use. Rebuild it after changing the
Dockerfile:

```bash
docker build -t mandate-ext-test-repo-base test-harness
```

## What isolation you get

The clone runs in a throwaway container as a non-root user, with hooks disabled
(`core.hooksPath=/dev/null`), local-path submodules blocked
(`protocol.file.allow=never`), submodules skipped, and all capabilities dropped.
That container is removed as soon as the clone finishes.

The container you keep is created stopped and never started by the script.
Nothing runs after the clone, so an idle repository costs no CPU or memory.

When you start it, it runs `tail -f /dev/null` as `quarantine` (uid 10001), with
`--network none`, `--cap-drop ALL`, `--security-opt no-new-privileges`, no host
mounts, and `/repo` mounted **read-only**. Nothing inside can modify the code it
is examining.

## What you do not get

- **A security guarantee.** Containers share the host kernel. This is a
  boundary, not a sandbox. Treat it as raising the cost of an accident, not as
  making one impossible.
- **An offline clone.** The clone container has network access, because cloning
  needs it. Only the container you keep is cut off.
- **Private repositories.** No credentials are passed. An authenticated URL fails
  rather than prompting.
- **Anything executed.** Nothing builds, installs, or tests the repository. Doing
  so inside the container voids everything above, because you would be running
  its code on purpose.
- **Updates.** To get newer commits, remove the container and volume and run
  again.
- **Shallow clones or a specific branch.** Full clone of the default branch only.
  Both are a flag away if they turn out to matter.

## Verified

Run end to end against Docker, not by inspection:

```
container state after run     created, not running
clone present at /repo        yes, spring-petclinic at 818c413
user                          quarantine, uid 10001
network mode                  none
capabilities                  all dropped
host mounts                   0
write to /repo                refused, read-only file system
second repository             reused the image, no rebuild
```
