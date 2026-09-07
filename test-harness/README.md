# test-harness

One script. Give it a repository URL, get a container with that repository
cloned inside it.

```bash
./run.sh https://github.com/spring-projects/spring-petclinic
```
```powershell
.\run.ps1 https://github.com/spring-projects/spring-petclinic
```

Output:

```
Container: mandate-docs-test-codebase-spring-petclinic (stopped)
Clone:     /repo

  docker start mandate-docs-test-codebase-spring-petclinic && docker exec -it mandate-docs-test-codebase-spring-petclinic sh
  docker rm -f mandate-docs-test-codebase-spring-petclinic
```

## Why

Third-party repositories are untrusted code. Cloning them onto the host puts
their hooks, submodules, and build scripts one careless command away from
running. A container keeps each one in its own box, and the predictable name
makes it obvious which is which.

One repository per container. No shared state between them.

## Naming

```
mandate-docs-test-codebase-<identifier>
```

The identifier defaults to the repository name from the URL. Pass a second
argument to override it:

```bash
./run.sh https://github.com/owner/name my-label
# mandate-docs-test-codebase-my-label
```

Either way it is lowercased, and anything outside `a-z0-9._-` becomes a dash.
The image takes the same name as the container. If the container already exists
the script stops rather than replacing it.

## Managing containers

There is no wrapper. These are plain Docker commands:

```bash
docker ps -a --filter "name=mandate-docs-test-codebase-"   # list them
docker start mandate-docs-test-codebase-<id>                # start one
docker exec -it mandate-docs-test-codebase-<id> sh          # shell in
docker stop mandate-docs-test-codebase-<id>                 # stop it again
docker rm -f mandate-docs-test-codebase-<id>                # remove one
docker rmi mandate-docs-test-codebase-<id>                  # remove its image
```

Remove every harness container and image at once:

```bash
docker rm -f $(docker ps -aq --filter "name=mandate-docs-test-codebase-")
docker rmi $(docker images -q "mandate-docs-test-codebase-*")
```

## What isolation you get

At build time the only thing that runs is `git clone`, with hooks disabled
(`core.hooksPath=/dev/null`), local-path submodules blocked
(`protocol.file.allow=never`), and submodules skipped entirely.

The container is created stopped and never started by the script. Nothing runs
after the clone finishes, so an idle repository costs no CPU or memory. Start it
when you want a shell, stop it when you are done.

When you do start it, it runs `tail -f /dev/null` as a non-root user, with
`--network none`, `--cap-drop ALL`, `--security-opt no-new-privileges`, and no
host mounts.

## What you do not get

- **A security guarantee.** Containers share the host kernel. This is a
  boundary, not a sandbox. Treat it as raising the cost of an accident, not as
  making one impossible.
- **An offline build.** Cloning needs the network, so the build step has it. Only
  the running container is cut off.
- **Private repositories.** No credentials are passed. An authenticated URL fails
  rather than prompting.
- **Anything executed.** Nothing builds, installs, or tests the repository. Doing
  so inside the container voids everything above, because you would be running
  its code on purpose.
- **Updates.** To get newer commits, remove the container and image and run again.
- **Shallow clones or a specific branch.** Full clone of the default branch only.
  Both are a `--build-arg` away if they turn out to matter.

## Status

Written but never executed. Docker was not available when this was built, so no
image has been built and no container started. Expect the first run to surface
something.
