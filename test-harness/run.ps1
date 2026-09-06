<#
.SYNOPSIS
Clone a third-party repository into an isolated container.

.DESCRIPTION
The identifier defaults to the repository name. The container is named
mandate-docs-test-codebase-<identifier> and the clone lives at /repo.

.EXAMPLE
.\run.ps1 https://github.com/owner/name
.EXAMPLE
.\run.ps1 https://github.com/owner/name my-copy -Ref v2.1.0 -Depth 1
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$RepoUrl,

    [Parameter(Position = 1)]
    [string]$Identifier = "",

    [string]$Ref = "",

    [int]$Depth = 0
)

$ErrorActionPreference = 'Stop'
$prefix = 'mandate-docs-test-codebase'

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error 'docker is not on PATH.'
    exit 1
}

if ($Depth -lt 0) {
    Write-Error "-Depth must be zero (full clone) or a positive integer."
    exit 2
}

# Derive the identifier from the last path segment of the URL when not given,
# then reduce whichever we have to lowercase alphanumerics, dashes and dots.
if ([string]::IsNullOrWhiteSpace($Identifier)) {
    $Identifier = ($RepoUrl.TrimEnd('/') -split '/')[-1]
    if ($Identifier.EndsWith('.git')) {
        $Identifier = $Identifier.Substring(0, $Identifier.Length - 4)
    }
}
$Identifier = $Identifier.ToLowerInvariant()
$Identifier = [regex]::Replace($Identifier, '[^a-z0-9._-]', '-')
$Identifier = $Identifier.Trim('.', '_', '-')

if ([string]::IsNullOrWhiteSpace($Identifier)) {
    Write-Error "Could not derive an identifier from '$RepoUrl'. Pass one explicitly."
    exit 2
}

$name = "$prefix-$Identifier"

$existing = @(docker ps -a --format '{{.Names}}')
if ($existing -contains $name) {
    Write-Host "A container named $name already exists."
    Write-Host ""
    Write-Host "  Re-enter it:  .\harness.ps1 shell $Identifier"
    Write-Host "  Remove it:    .\harness.ps1 rm $Identifier"
    Write-Host ""
    Write-Host "Or pass a different identifier as the second argument."
    exit 1
}

$here = $PSScriptRoot
$depthArg = ''
if ($Depth -gt 0) { $depthArg = "$Depth" }

Write-Host "Building image $name from $RepoUrl"
docker build `
    --pull `
    --build-arg "REPO_URL=$RepoUrl" `
    --build-arg "REPO_REF=$Ref" `
    --build-arg "CLONE_DEPTH=$depthArg" `
    --label 'org.mandate-docs.harness=test-codebase' `
    -t $name `
    -f (Join-Path $here 'Dockerfile') `
    $here
if ($LASTEXITCODE -ne 0) { Write-Error 'docker build failed.'; exit $LASTEXITCODE }

Write-Host "Starting container $name"
docker run -d `
    --name $name `
    --label 'org.mandate-docs.harness=test-codebase' `
    --network none `
    --cap-drop ALL `
    --security-opt no-new-privileges `
    --pids-limit 512 `
    --memory 2g `
    $name | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Error 'docker run failed.'; exit $LASTEXITCODE }

Write-Host ""
Write-Host "Container: $name"
Write-Host "Clone:     /repo (inside the container)"
Write-Host "Network:   disabled"
Write-Host "Mounts:    none"
Write-Host ""
Write-Host "Shell in:  .\harness.ps1 shell $Identifier"
Write-Host "           docker exec -it $name sh"
Write-Host "Remove:    .\harness.ps1 rm $Identifier"
