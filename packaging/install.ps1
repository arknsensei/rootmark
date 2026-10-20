# Installs a rootmark release on Windows x86_64 (P2-A/A3; docs/RELEASE.md
# "Kurulum"): downloads the release zip and SHA256SUMS, checks the zip's
# SHA-256 against SHA256SUMS, installs rootmark.exe and the embedder model
# manifest, and fetches the pinned model files, each checked against
# models/MANIFEST.json (size and SHA-256). Any mismatch or missing file is
# an error; nothing is installed until every check passed.
#
#   install.ps1 [-Version <x.y.z>] [-Prefix <dir>] [-From <dir>] [-NoModel]
#
#   -Version  the release (default: the latest GitHub release)
#   -Prefix   install directory (default: $env:LOCALAPPDATA\rootmark)
#   -From     a directory holding the zip and SHA256SUMS instead of the
#             GitHub release (CI checks the artefacts this way)
#   -NoModel  do not download the model files (search needs them)
#
# Provenance: `gh attestation verify <zip> -R arknsensei/rootmark`.
# The binary is not code-signed (Authenticode is a human step,
# docs/RELEASE.md).
param(
  [string]$Version = "",
  [string]$Prefix = (Join-Path $env:LOCALAPPDATA "rootmark"),
  [string]$From = "",
  [switch]$NoModel
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$repo = "arknsensei/rootmark"
$target = "x86_64-pc-windows-msvc"
function Die($m) { Write-Error "install.ps1: $m"; exit 1 }

if (-not [Environment]::Is64BitOperatingSystem -or $env:PROCESSOR_ARCHITECTURE -ne "AMD64") {
  Die "no release archive for $($env:PROCESSOR_ARCHITECTURE) (x86_64 only)"
}
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("rootmark-install-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
  function Fetch($name) {
    $out = Join-Path $tmp $name
    if ($From -ne "") {
      $src = Join-Path $From $name
      if (-not (Test-Path $src)) { Die "$src missing" }
      Copy-Item $src $out
    } else {
      Invoke-WebRequest -UseBasicParsing -Uri "https://github.com/$repo/releases/download/v$Version/$name" -OutFile $out
    }
    return $out
  }
  if ($Version -eq "") {
    if ($From -ne "") {
      $z = Get-ChildItem -Path $From -Filter "rootmark-*-$target.zip" | Select-Object -First 1
      if ($null -eq $z) { Die "no rootmark-*-$target.zip in $From" }
      $Version = $z.Name.Substring(9, $z.Name.Length - 9 - ("-$target.zip").Length)
    } else {
      $Version = (Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest").tag_name.TrimStart("v")
    }
  }
  $zipName = "rootmark-$Version-$target.zip"
  $zip = Fetch $zipName
  $sums = Fetch "SHA256SUMS"
  $want = $null
  foreach ($line in Get-Content $sums) {
    $parts = $line -split "\s+", 2
    if ($parts.Count -eq 2 -and $parts[1].TrimStart("*") -eq $zipName) { $want = $parts[0].ToLower() }
  }
  if ($null -eq $want) { Die "$zipName is not listed in SHA256SUMS" }
  $got = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower()
  if ($got -ne $want) { Die "$zipName SHA-256 $got != SHA256SUMS $want" }
  Expand-Archive -Path $zip -DestinationPath $tmp
  $d = Join-Path $tmp "rootmark-$Version-$target"
  $exe = Join-Path $d "rootmark.exe"
  $manifest = Join-Path $d "models\MANIFEST.json"
  if (-not (Test-Path $exe) -or -not (Test-Path $manifest)) { Die "the archive does not hold rootmark.exe and models\MANIFEST.json" }
  $v = (& $exe --version).Trim()
  if ($v -ne "rootmark $Version") { Die "the archive's binary says '$v', not 'rootmark $Version'" }

  $models = Join-Path $Prefix "models"
  New-Item -ItemType Directory -Force -Path $models | Out-Null
  Copy-Item $exe (Join-Path $Prefix "rootmark.exe") -Force
  Copy-Item $manifest (Join-Path $models "MANIFEST.json") -Force
  Copy-Item (Join-Path $d "LICENSE") (Join-Path $Prefix "LICENSE") -Force
  if (-not $NoModel) {
    $m = Get-Content $manifest -Raw | ConvertFrom-Json
    $dest = Join-Path $models $m.name
    foreach ($p in $m.files.PSObject.Properties) {
      $f = $p.Value
      $out = Join-Path $dest $p.Name
      New-Item -ItemType Directory -Force -Path (Split-Path $out) | Out-Null
      if ((Test-Path $out) -and ((Get-FileHash -Algorithm SHA256 $out).Hash.ToLower() -eq $f.sha256)) { continue }
      $part = Join-Path $tmp ("model-" + [Guid]::NewGuid().ToString("N"))
      Invoke-WebRequest -UseBasicParsing -Uri "https://huggingface.co/$($m.source)/resolve/$($m.revision)/$($f.remote_path)" -OutFile $part
      if ((Get-Item $part).Length -ne [int64]$f.size) { Die "$($p.Name): size $((Get-Item $part).Length) != manifest $($f.size)" }
      $h = (Get-FileHash -Algorithm SHA256 $part).Hash.ToLower()
      if ($h -ne $f.sha256) { Die "$($p.Name): SHA-256 $h != manifest $($f.sha256)" }
      Move-Item $part $out -Force
    }
  }
  Write-Output "install.ps1: rootmark $Version -> $(Join-Path $Prefix 'rootmark.exe') (SHA-256 checked against SHA256SUMS)"
  Write-Output "install.ps1: set ROOTMARK_MODELS=$models (search and serve need the model files)"
} finally {
  Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
