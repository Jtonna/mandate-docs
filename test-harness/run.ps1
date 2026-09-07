# Clone a repository into an isolated container.
#
#   .\run.ps1 <repo-url> [identifier]
#
# The identifier defaults to the repository name. The container is named
# mandate-docs-test-codebase-<identifier> and the clone lives at /repo.

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$RepoUrl,

    [Parameter(Position = 1)]
    [string]$Identifier
)

$ErrorActionPreference = 'Stop'
$prefix = 'mandate-docs-test-codebase'

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error 'docker is not on PATH.'
    exit 1
}

# Derive the identifier from the last path segment of the URL when not given,
# then reduce it to lowercase characters Docker accepts in a name.
if ([string]::IsNullOrWhiteSpace($Identifier)) {
    $Identifier = ($RepoUrl.TrimEnd('/') -split '/')[-1] -replace '\.git$', ''
}
$Identifier = $Identifier.ToLowerInvariant() -replace '[^a-z0-9._-]', '-'
$Identifier = $Identifier -replace '^[._-]+', '' -replace '[._-]+$', ''

if ([string]::IsNullOrWhiteSpace($Identifier)) {
    Write-Error "Could not derive an identifier from '$RepoUrl'. Pass one explicitly."
    exit 2
}

$name = "$prefix-$Identifier"

$existing = docker ps -a --format '{{.Names}}'
if ($existing -contains $name) {
    Write-Error "A container named $name already exists. Remove it with: docker rm -f $name"
    exit 1
}

$here = Split-Path -Parent $MyInvocation.MyCommand.Path

docker build --build-arg "REPO_URL=$RepoUrl" -t $name -f (Join-Path $here 'Dockerfile') $here
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# create, not run. The clone already happened at build time, so the container
# has nothing to do until you want a shell in it. It is left stopped.
docker create --name $name --network none --cap-drop ALL --security-opt no-new-privileges $name tail -f /dev/null | Out-Null
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ''
Write-Host "Container: $name (stopped)"
Write-Host 'Clone:     /repo'
Write-Host ''
Write-Host "  docker start $name; docker exec -it $name sh"
Write-Host "  docker rm -f $name"
