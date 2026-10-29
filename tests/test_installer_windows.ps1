# Run with PowerShell 5.1+ or pwsh. Loads helpers only; never installs anything.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Split-Path $PSScriptRoot -Parent
$tokens = $null
$errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile(
    (Join-Path $root 'install.ps1'), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors | Out-String) }
foreach ($name in @('Initialize-InstallationMarker', 'Remove-TemporaryBuildFiles')) {
    $definition = $ast.Find({ param($node)
        $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq $name
    }, $true)
    . ([scriptblock]::Create($definition.Extent.Text))
}
$temporary = Join-Path ([IO.Path]::GetTempPath()) ('leditor-script-test-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory $temporary | Out-Null
try {
    $base = Join-Path $temporary 'app'
    New-Item -ItemType Directory $base | Out-Null
    Initialize-InstallationMarker $base
    $marker = Join-Path $base '.leditor-managed'
    if ((Get-Content -LiteralPath $marker -Raw).Trim() -ne 'leditor-installer-v1') { throw 'Marker was not created' }
    Write-Host 'PASS: new installation marker'
    $before = [IO.File]::GetLastWriteTimeUtc($marker)
    # On Windows this sharing mode permits readers but refuses another writer.
    $reader = [IO.File]::Open($marker, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try { Initialize-InstallationMarker $base } finally { $reader.Dispose() }
    if ([IO.File]::GetLastWriteTimeUtc($marker) -ne $before) { throw 'Existing marker was rewritten' }
    Write-Host 'PASS: existing marker is not rewritten'
    $work = Join-Path $temporary 'build [temporary]'
    New-Item -ItemType Directory (Join-Path $work 'source\target') -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $work 'source\target\binary') 'temporary'
    $messages = Remove-TemporaryBuildFiles $work 6>&1 | Out-String
    if (Test-Path -LiteralPath $work) { throw 'Temporary build directory remains' }
    if (!(Test-Path -LiteralPath $marker)) { throw 'Cleanup removed files outside the build directory' }
    if ($messages -notmatch 'Removing: source' -or $messages -notmatch 'Temporary build files removed') { throw 'Cleanup progress is missing' }
    Write-Host 'PASS: cleanup removes temporary files and reports progress'
} finally {
    Remove-Item -LiteralPath $temporary -Recurse -Force
}
