# One-line install for Windows PowerShell:
#   irm https://raw.githubusercontent.com/AkaraChen/para/main/scripts/install.ps1 | iex
[CmdletBinding()]
param(
    [string]$Repo = $(if ($env:PARA_REPO) { $env:PARA_REPO } else { "AkaraChen/para" }),
    [string]$Bin = $(if ($env:PARA_BIN) { $env:PARA_BIN } else { "para" }),
    [string]$Prefix = $(if ($env:PARA_PREFIX) { $env:PARA_PREFIX } else { Join-Path $env:LOCALAPPDATA "para\bin" }),
    [string]$Version = $(if ($env:PARA_VERSION) { $env:PARA_VERSION } else { "" })
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-ParaArch {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" { return "amd64" }
        "ARM64" { return "arm64" }
        default { throw "unsupported architecture: $($env:PROCESSOR_ARCHITECTURE)" }
    }
}

if (-not $Version) {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $release.tag_name
}
if (-not $Version) {
    throw "could not resolve the latest GitHub release for $Repo"
}
$Version = $Version.TrimStart("v")

$arch = Get-ParaArch
$asset = "${Bin}_${Version}_windows_${arch}.zip"
$base = "https://github.com/$Repo/releases/download/v$Version"
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("para-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tmp | Out-Null

try {
    Write-Host "installing $Bin v$Version (windows/$arch) to $Prefix"
    $zipPath = Join-Path $tmp $asset
    $sumPath = Join-Path $tmp "checksums.txt"
    Invoke-WebRequest -Uri "$base/$asset" -OutFile $zipPath -UseBasicParsing
    Invoke-WebRequest -Uri "$base/checksums.txt" -OutFile $sumPath -UseBasicParsing

    $expected = (Get-Content $sumPath | Where-Object { $_ -match [regex]::Escape($asset) } | Select-Object -First 1)
    if ($expected) {
        $want = ($expected -split "\s+")[0].ToLowerInvariant()
        $hash = (Get-FileHash -Algorithm SHA256 -Path $zipPath).Hash.ToLowerInvariant()
        if ($hash -ne $want) {
            throw "checksum mismatch for $asset"
        }
    } else {
        Write-Warning "checksums.txt has no entry for $asset"
    }

    Expand-Archive -Path $zipPath -DestinationPath $tmp -Force
    $src = Get-ChildItem -Path $tmp -Recurse -Filter "$Bin.exe" | Select-Object -First 1
    if (-not $src) {
        throw "archive did not contain $Bin.exe"
    }

    New-Item -ItemType Directory -Force -Path $Prefix | Out-Null
    $dest = Join-Path $Prefix "$Bin.exe"
    Copy-Item -Force -Path $src.FullName -Destination $dest

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $userPath) { $userPath = "" }
    $parts = $userPath -split ";" | Where-Object { $_ -ne "" }
    if ($parts -notcontains $Prefix) {
        [Environment]::SetEnvironmentVariable("Path", ($parts + $Prefix) -join ";", "User")
        $env:Path = "$Prefix;$env:Path"
        Write-Host "added $Prefix to the user PATH (open a new terminal if $Bin is not found)"
    }

    & $dest --help | Out-Null
    Write-Host "installed $dest"
} finally {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmp
}
