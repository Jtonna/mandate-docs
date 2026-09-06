# test-harness

A way to clone a third-party repository into a Docker container so it can be
read and reasoned about without the code ever touching the host.

One repository per container. The clone lives at `/repo` inside the container,
and nothing from it is ever executed, at build time or at run time.

This replaces cloning repositories into `test_repos/` on the host, where any
build script, hook, or editor plugin the repository carries would be running
with your account's access to your machine.

## What is in here

```
Dockerfile      the quarantine image: git, a shell, a non-root user, a clone
run.sh          launch a container from a repository URL (Git Bash)
run.ps1         the same, for PowerShell
harness.sh      list, shell, rm, rm-all (Git Bash)
harness.ps1     the same, for PowerShell
```

## Launching a container

From this folder, in Git Bash:

```bash
./run.sh https://github.com/owner/name
```

or in PowerShell:

```powershell
.\run.ps1 https://github.com/owner/name
```

That builds an image, clones the repository into it, and starts a container
named `mandate-docs-test-codebase-name`. The script prints the container name
and how to shell into it.

The second argument is an identifier, used as the container name suffix. It
defaults to the repository name from the URL. Pass one when two repositories
share a name, or when you want two copies of the same repository:

```bash
./run.sh https://github.com/owner/name upstream
./run.sh https://github.com/fork/name  fork
```

Identifiers are lowercased and anything outside `a-z0-9._-` becomes a dash, so
the container name is always a legal Docker name.

Two optional flags:

```bash
./run.sh https://github.com/owner/name --ref v2.1.0 --depth 1
```

```powershell
.\run.ps1 https://github.com/owner/name -Ref v2.1.0 -Depth 1
```

`--ref` is a branch or tag, defaulting to whatever the remote calls its default
branch. `--depth` is a shallow clone depth, defaulting to a full clone.

If a container with that name already exists the script refuses to touch it and
tells you how to re-enter or remove it. Nothing is ever clobbered.

## Lifecycle

```bash
./harness.sh list             # every harness container and its state
./harness.sh shell <id>       # shell into one, starting it if stopped
./harness.sh rm <id>          # remove the container and its image
./harness.sh rm-all           # remove all of them
```

```powershell
.\harness.ps1 list
.\harness.ps1 shell <id>
.\harness.ps1 rm <id>
.\harness.ps1 rm-all
```

`<id>` is the identifier, not the full container name. The shell opens in
`/repo` as the non-root user `quarantine`.

Containers are found by the label `org.mandate-docs.harness=test-codebase`, so
`list` and `rm-all` only ever see containers this harness created.

## What the isolation actually is

At build time, the only thing that runs is `git clone`. No package manager
reads the repository's manifests, no install script runs, no build step runs.
Git hooks are not fetched by a clone, and `core.hooksPath` is pointed at
`/dev/null` anyway. Submodules are not fetched.

At run time the container runs `tail -f /dev/null` and nothing else. It waits
for you to shell in.

The container is started with:

- `--network none`, so it has no network at all. The clone already happened
  during the build, so nothing needs one afterwards.
- No volume or bind mounts, so no path on the host is visible inside.
- `--cap-drop ALL` and `--security-opt no-new-privileges`.
- A non-root user, UID 10001.
- `--pids-limit 512` and `--memory 2g`.

## Limitations

A container is a boundary, not a guarantee. It shares the host kernel, and a
kernel exploit or a Docker escape defeats all of the above. Treat this as
raising the cost of an attack, not as a sandbox you would run known-malicious
code in.

Other things this does not handle:

- **The build has network access.** It has to, in order to clone. A malicious
  URL is reaching the network during that step, though only git is running.
- **Private repositories are not supported.** No credentials are passed in, and
  `GIT_TERMINAL_PROMPT=0` makes an authenticated URL fail rather than hang.
- **The root filesystem is writable.** Read-only would break ordinary git
  commands inside the container. Anything written is lost when the container is
  removed, which is the intent.
- **Nothing runs the repository's tests.** This is a reading environment. The
  moment you execute code from `/repo`, the guarantees in the section above
  stop describing what is happening.
- **Images are not shared between containers.** Each repository builds its own
  image, named the same as its container. The base layer is shared, the clone
  layer is not.
- **No update path.** To pull newer commits, remove the container and run the
  script again.

## Requirements

Docker Desktop on Windows 11, with either Git Bash or PowerShell. WSL is not
required. In Git Bash, `docker exec -it` sometimes needs a `winpty` prefix if
you call it directly; `harness.sh shell` is the supported route.
