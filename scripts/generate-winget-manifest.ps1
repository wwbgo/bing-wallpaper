param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$InstallerUrl,

    [Parameter(Mandatory = $true)]
    [string]$InstallerSha256,

    [string]$OutputRoot = "dist/winget",

    [string]$Repository = "wwbgo/bing-wallpaper"
)

$ErrorActionPreference = "Stop"

$identifier = "wwb.BingWallpaper"
$versionDir = Join-Path $OutputRoot "manifests/w/wwb/BingWallpaper/$Version"
New-Item -ItemType Directory -Force -Path $versionDir | Out-Null

$releaseNotesUrl = "https://github.com/$Repository/releases/tag/v$Version"
$releaseDate = Get-Date -Format "yyyy-MM-dd"
$InstallerSha256 = $InstallerSha256.ToUpperInvariant()
$publisherUrl = "https://github.com/$Repository"
$supportUrl = "https://github.com/$Repository/issues"

$versionManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.version.1.12.0.schema.json

PackageIdentifier: $identifier
PackageVersion: $Version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.12.0
"@

$defaultLocaleManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.defaultLocale.1.12.0.schema.json

PackageIdentifier: $identifier
PackageVersion: $Version
PackageLocale: en-US
Publisher: wwb
PublisherUrl: $publisherUrl
PublisherSupportUrl: $supportUrl
Author: wwb
PackageName: Bing Wallpaper
PackageUrl: $publisherUrl
License: Proprietary
Copyright: Copyright (c) wwb
ShortDescription: Lightweight daily Bing wallpaper updater for Windows.
Description: Downloads the Bing image of the day and sets it as the Windows desktop wallpaper. Supports scheduled updates, multiple wallpaper styles, and automatic resolution selection.
Moniker: bing-wallpaper
Tags:
  - bing
  - wallpaper
  - desktop
  - daily
  - windows
ReleaseNotesUrl: $releaseNotesUrl
ManifestType: defaultLocale
ManifestVersion: 1.12.0
"@

$installerManifest = @"
# yaml-language-server: `$schema=https://aka.ms/winget-manifest.installer.1.12.0.schema.json

PackageIdentifier: $identifier
PackageVersion: $Version
Platform:
  - Windows.Desktop
MinimumOSVersion: 10.0.17763.0
InstallerType: inno
Scope: user
InstallModes:
  - silent
  - silentWithProgress
UpgradeBehavior: install
ReleaseDate: $releaseDate
Installers:
  - Architecture: x64
    InstallerUrl: $InstallerUrl
    InstallerSha256: $InstallerSha256
    InstallerSwitches:
      Silent: /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-
      SilentWithProgress: /SILENT /SUPPRESSMSGBOXES /NORESTART /SP-
ManifestType: installer
ManifestVersion: 1.12.0
"@

$versionManifest | Set-Content -Path (Join-Path $versionDir "wwb.BingWallpaper.yaml") -Encoding ascii
$defaultLocaleManifest | Set-Content -Path (Join-Path $versionDir "wwb.BingWallpaper.locale.en-US.yaml") -Encoding ascii
$installerManifest | Set-Content -Path (Join-Path $versionDir "wwb.BingWallpaper.installer.yaml") -Encoding ascii

Write-Output "generated winget manifests in $versionDir"
