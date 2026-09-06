<#
.SYNOPSIS
Lifecycle commands for the quarantine containers created by run.ps1.

.EXAMPLE
.\harness.ps1 list
.EXAMPLE
.\harness.ps1 shell my-copy
.EXAMPLE
.\harness.ps1 rm my-copy
.EXAMPLE
.\harness.ps1 rm-all
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('list', 'shell', 'rm', 'rm-all')]
    [string]$Command,

    [Parameter(Position = 1)]
    [string]$Identifier = ""
)

$ErrorActionPreference = 'Stop'
$prefix = 'mandate-docs-test-codebase'
$label = 'org.mandate-docs.harness=test-codebase'

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error 'docker is not on PATH.'
    exit 1
}

function Get-HarnessName {
    param([string]$Id)
    if ([string]::IsNullOrWhiteSpace($Id)) {
        Write-Error "This command needs an identifier, for example: .\harness.ps1 $Command my-copy"
        exit 2
    }
    return "$prefix-$Id"
}

function Assert-Container {
    param([string]$Name)
    $all = @(docker ps -a --format '{{.Names}}')
    if (-not ($all -contains $Name)) {
        Write-Error "No container named $Name. Run .\harness.ps1 list to see what exists."
        exit 1
    }
}

function Remove-Harness {
    param([string]$Name)
    docker rm -f $Name | Out-Null
    docker image rm -f $Name 2>$null | Out-Null
    Write-Host "Removed $Name"
}

switch ($Command) {
    'list' {
        docker ps -a --filter "label=$label" --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}'
    }

    'shell' {
        $name = Get-HarnessName $Identifier
        Assert-Container $name
        $running = docker inspect -f '{{.State.Running}}' $name
        if ($running -ne 'true') { docker start $name | Out-Null }
        docker exec -it -w /repo $name sh
    }

    'rm' {
        $name = Get-HarnessName $Identifier
        Assert-Container $name
        Remove-Harness $name
    }

    'rm-all' {
        $names = @(docker ps -a --filter "label=$label" --format '{{.Names}}')
        $names = $names | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
        if ($names.Count -eq 0) {
            Write-Host 'No harness containers to remove.'
            return
        }
        foreach ($n in $names) { Remove-Harness $n }
    }
}
