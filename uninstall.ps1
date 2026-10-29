#Requires -Version 5.1
param([ValidateSet('user','system')][string]$Scope)
$ErrorActionPreference = 'Stop'
if (!$Scope) { $Scope = if ((Read-Host 'Remove [1] user (default) or [2] system installation') -eq '2') {'system'} else {'user'} }
$admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($Scope -eq 'system' -and !$admin) {
    $p = Start-Process powershell.exe -Verb RunAs -ArgumentList @('-NoProfile','-File',('"' + $PSCommandPath + '"'),'-Scope','system') -Wait -PassThru
    exit $p.ExitCode
}
$base = if ($Scope -eq 'system') { Join-Path $env:ProgramFiles 'Leditor' } else { Join-Path $env:LOCALAPPDATA 'Programs\Leditor' }
if (!(Test-Path "$base\.leditor-managed") -or (Get-Content "$base\.leditor-managed" -Raw).Trim() -ne 'leditor-installer-v1') { throw "No managed installation: $base" }
if ((Get-Item $base).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Refusing a junction installation directory.' }
if (Test-Path "$base\.install-lock") { throw 'An installation may be active. Close it before uninstalling.' }
Write-Host "Close Leditor. This removes $base."
if ((Read-Host 'Type REMOVE to confirm') -cne 'REMOVE') { exit 0 }
$start = if ($Scope -eq 'system') { [Environment]::GetFolderPath('CommonPrograms') } else { [Environment]::GetFolderPath('Programs') }
$shortcut = Join-Path $start 'Leditor.lnk'
if (Test-Path $shortcut) {
    $shell = New-Object -ComObject WScript.Shell
    if ($shell.CreateShortcut($shortcut).TargetPath.StartsWith($base + '\', [StringComparison]::OrdinalIgnoreCase)) { Remove-Item -LiteralPath $shortcut }
}
Remove-Item -LiteralPath $base -Recurse -Force
if ((Read-Host 'Remove settings and recovered/unsaved documents for this account? [y/N]') -match '^[yYsS]$') {
    foreach ($parent in @($env:LOCALAPPDATA, $env:APPDATA)) {
        $path = Join-Path $parent 'leditor'
        if ((Test-Path $path) -and !((Get-Item $path).Attributes -band [IO.FileAttributes]::ReparsePoint)) { Remove-Item -LiteralPath $path -Recurse -Force }
    }
}
Write-Host 'Leditor removed. Shared tools and other users were not modified.'
