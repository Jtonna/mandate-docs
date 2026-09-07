# Clone a repository into an isolated container.
#
#   .\run.ps1 <repo-url> [identifier]
#
# The identifier defaults to the owner and repository from the URL. The
# container is named mandate-ext-repo-test-<identifier> and the clone lives
# at /repo.

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$RepoUrl,

    [Parameter(Position = 1)]
    [string]$Identifier
)

$ErrorActionPreference = 'Stop'
$prefix = 'mandate-ext-repo-test'

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error 'docker is not on PATH.'
    exit 1
}

# Derive the identifier from the owner and repository in the URL when not given.
# Docker names cannot contain a slash or colon, so the scheme and host are
# dropped and the remaining path segments are joined with dashes.
if ([string]::IsNullOrWhiteSpace($Identifier)) {
    $Identifier = $RepoUrl.TrimEnd('/') -replace '\.git$', ''
    $Identifier = $Identifier -replace '^[a-zA-Z][a-zA-Z0-9+.-]*://', ''
    $Identifier = $Identifier -replace '^[^/@]*@', ''
    $Identifier = $Identifier -replace '^[^/:]*[/:]', ''
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
