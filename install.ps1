#Requires -Version 5.1
[CmdletBinding()]
param([ValidateSet('user','system')][string]$Scope, [string]$Source, [string]$Ref = 'main')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
function Invoke-Checked([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed (exit $LASTEXITCODE)" }
}
function Download([string]$Url, [string]$Destination) {
    Write-Host "Downloading $Url"
    Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $Destination
}
function Initialize-InstallationMarker([string]$Directory) {
    $marker = Join-Path $Directory '.leditor-managed'
    if (!(Test-Path -LiteralPath $marker)) {
        Set-Content -LiteralPath $marker 'leditor-installer-v1'
    }
}
function Copy-PdfiumLicenses([string]$ArchiveDirectory, [string]$Destination) {
    # Bundled notices already exist; merge notices from the actual runtime.
    New-Item -ItemType Directory -Force $Destination | Out-Null
    Get-ChildItem -LiteralPath $ArchiveDirectory | Where-Object { $_.Name -match '^(LICENSE|NOTICE)' } |
        Copy-Item -Destination $Destination -Recurse -Force
}
function Remove-TemporaryBuildFiles([string]$Directory) {
    Write-Host 'Removing temporary build files and tools. Large folders may take several minutes.'
    foreach ($item in Get-ChildItem -LiteralPath $Directory -Force) {
        Write-Host "  Removing: $($item.Name)"
        Remove-Item -LiteralPath $item.FullName -Recurse -Force
    }
    Remove-Item -LiteralPath $Directory -Force
    Write-Host 'Temporary build files removed.'
}
if ([Environment]::OSVersion.Platform -ne 'Win32NT' -or $env:PROCESSOR_ARCHITECTURE -ne 'AMD64') {
    throw 'Supported: Windows 10/11 x64, running 64-bit PowerShell.'
}
if (!$Scope) { $Scope = if ((Read-Host 'Install for [1] current user (default) or [2] all users') -eq '2') {'system'} else {'user'} }
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($Scope -eq 'system' -and !$isAdmin) {
    # Only this explicitly selected branch requests UAC elevation.
    $argsList = @('-NoProfile','-File',('"' + $PSCommandPath + '"'),'-Scope','system','-Ref',('"' + $Ref + '"'))
    if ($Source) { $argsList += @('-Source',('"' + (Resolve-Path $Source).Path + '"')) }
    $p = Start-Process powershell.exe -Verb RunAs -ArgumentList $argsList -Wait -PassThru
    exit $p.ExitCode
}
$base = if ($Scope -eq 'system') { Join-Path $env:ProgramFiles 'Leditor' } else { Join-Path $env:LOCALAPPDATA 'Programs\Leditor' }
$start = if ($Scope -eq 'system') { [Environment]::GetFolderPath('CommonPrograms') } else { [Environment]::GetFolderPath('Programs') }
if (Test-Path $base) {
    if (!(Test-Path "$base\.leditor-managed") -or (Get-Content "$base\.leditor-managed" -Raw).Trim() -ne 'leditor-installer-v1') { throw "Unmanaged directory: $base" }
    if ((Get-Item $base).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Installation directory must not be a junction.' }
}
$work = Join-Path ([IO.Path]::GetTempPath()) ('leditor-build-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory $work | Out-Null
$generation = $null
$committed = $false
$oldPath = $env:PATH
$oldCargo = $env:CARGO_HOME
$oldTarget = $env:CARGO_TARGET_DIR
$oldMsystem = $env:MSYSTEM
$oldChere = $env:CHERE_INVOKING
$oldJobs = $env:CARGO_BUILD_JOBS
if (!$env:CARGO_BUILD_JOBS) { $env:CARGO_BUILD_JOBS = '2' }
$installLock = $null
try {
    New-Item -ItemType Directory -Force $base | Out-Null
    $installLock = [IO.File]::Open("$base\.install-lock", [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    Initialize-InstallationMarker $base
    # Reuse a complete UCRT64 installation without changing its packages.
    $msys = $null
    foreach ($candidate in @($env:LEDITOR_MSYS2, 'C:\msys64')) {
        if ($candidate -and (Test-Path "$candidate\ucrt64\bin\cargo.exe") -and (Test-Path "$candidate\ucrt64\bin\libgtksourceview-5-0.dll") -and (Test-Path "$candidate\ucrt64\bin\python.exe") -and (Test-Path "$candidate\usr\bin\git.exe") -and (Test-Path "$candidate\ucrt64\bin\objdump.exe") -and (Test-Path "$candidate\ucrt64\bin\glib-compile-resources.exe")) {
            $rustVersion = & "$candidate\ucrt64\bin\rustc.exe" --version
            if ($LASTEXITCODE -eq 0 -and $rustVersion -match 'rustc (\d+\.\d+\.\d+)') {
                if ([version]$Matches[1] -ge [version]'1.88.0') { $msys = $candidate; break }
            }
        }
    }
    if (!$msys) {
        $archive = Join-Path $work 'msys.tar.xz'
        Download 'https://repo.msys2.org/distrib/x86_64/msys2-base-x86_64-20260927.tar.xz' $archive
        Invoke-Checked 'tar.exe' @('-xf', $archive, '-C', $work)
        $msys = Join-Path $work 'msys64'
        $env:MSYSTEM = 'UCRT64'
        $env:CHERE_INVOKING = '1'
        # Complete system upgrades before installing packages; never leave a partial upgrade.
        Invoke-Checked "$msys\usr\bin\bash.exe" @('-lc', 'pacman -Syu --noconfirm')
        Invoke-Checked "$msys\usr\bin\bash.exe" @('-lc', 'pacman -Syu --noconfirm')
        Invoke-Checked "$msys\usr\bin\bash.exe" @('-lc', 'pacman -S --needed --noconfirm git mingw-w64-ucrt-x86_64-rust mingw-w64-ucrt-x86_64-gcc mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-gtksourceview5 mingw-w64-ucrt-x86_64-pkgconf mingw-w64-ucrt-x86_64-python')
    }
    $ucrt = Join-Path $msys 'ucrt64'
    $env:PATH = "$ucrt\bin;$msys\usr\bin;$oldPath"
    $env:CARGO_HOME = Join-Path $work 'cargo'
    $cargo = "$ucrt\bin\cargo.exe"
    $sourceDir = Join-Path $work 'source'
    $env:CARGO_TARGET_DIR = Join-Path $sourceDir 'target'
    if ($Source) {
        $origin = (Resolve-Path $Source).Path
        New-Item -ItemType Directory $sourceDir | Out-Null
        Get-ChildItem -Force $origin | Where-Object { $_.Name -notin @('.git','target') } | Copy-Item -Destination $sourceDir -Recurse
        $revision = 'local-' + [guid]::NewGuid().ToString('N')
    } else {
        Invoke-Checked "$msys\usr\bin\git.exe" @('clone','--depth','1','--branch',$Ref,'https://github.com/Bartzin55/Leditor.git',$sourceDir)
        $revision = (& "$msys\usr\bin\git.exe" -C $sourceDir rev-parse HEAD).Trim()
    }
    if (!$Source -and (Test-Path "$base\revision") -and (Get-Content "$base\revision" -Raw).Trim() -eq $revision) { Write-Host 'Already up to date.'; return }
    Push-Location $sourceDir
    try {
        Invoke-Checked $cargo @('build','--locked','--release')
        Invoke-Checked $cargo @('test','--locked','--lib','--test','document','--test','file_ops')
        Write-Host 'Tests passed. Preparing application files and dependencies...'
        New-Item -ItemType Directory -Force "$base\releases" | Out-Null
        $generation = Join-Path "$base\releases" ([guid]::NewGuid().ToString('N'))
        New-Item -ItemType Directory $generation | Out-Null
        Copy-Item 'target\release\leditor.exe' $generation
        Invoke-Checked "$ucrt\bin\python.exe" @('scripts\bundle_windows.py', $ucrt, $generation)
        if (Test-Path "$ucrt\bin\gdk-pixbuf-query-loaders.exe") {
            Copy-Item "$ucrt\bin\gdk-pixbuf-query-loaders.exe" $generation
            $loaders = @(Get-ChildItem "$generation\gdk-pixbuf-2.0" -Filter '*.dll' -Recurse | ForEach-Object { $_.FullName })
            if ($loaders.Count -gt 0) {
                $cache = & "$generation\gdk-pixbuf-query-loaders.exe" @loaders
                if ($LASTEXITCODE -ne 0) { throw 'Image loader cache generation failed' }
                [IO.File]::WriteAllLines("$generation\loaders.cache", [string[]]$cache, (New-Object Text.UTF8Encoding($false)))
            }
        }
        Copy-Item LICENSES "$generation\LICENSES" -Recurse
        Invoke-Checked "$ucrt\bin\python.exe" @('scripts\collect_licenses.py',"$generation\LICENSES\rust")
        Copy-Item "$ucrt\share\licenses" "$generation\LICENSES\native" -Recurse
        & "$msys\usr\bin\pacman.exe" -Q | Set-Content "$generation\LICENSES\native-packages.txt"
        $pdf = Join-Path $work 'pdfium'
        New-Item -ItemType Directory $pdf | Out-Null
        Download 'https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7881/pdfium-win-x64.tgz' "$work\pdfium.tgz"
        if ((Get-FileHash "$work\pdfium.tgz" -Algorithm SHA256).Hash -ne '73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08') { throw 'PDFium checksum mismatch' }
        Invoke-Checked 'tar.exe' @('-xf',"$work\pdfium.tgz",'-C',$pdf)
        Copy-Item "$pdf\bin\pdfium.dll" $generation
        Copy-PdfiumLicenses $pdf "$generation\LICENSES\pdfium"
        # Launcher paths are relative, so moving the installation does not break GTK data lookup.
        $launch = "@echo off`r`nset `"XDG_DATA_DIRS=%~dp0share`"`r`nset `"GSETTINGS_SCHEMA_DIR=%~dp0share\glib-2.0\schemas`"`r`n"
        if (Test-Path "$generation\loaders.cache") { $launch += "set `"GDK_PIXBUF_MODULE_FILE=%~dp0loaders.cache`"`r`n" }
        $launch += "start `"`" `"%~dp0leditor.exe`" %*`r`n"
        [IO.File]::WriteAllText("$generation\launch.cmd", $launch, [Text.Encoding]::ASCII)
        Copy-Item uninstall.ps1 "$base\uninstall.ps1"
        $shell = New-Object -ComObject WScript.Shell
        $shortcutPath = Join-Path $start 'Leditor.lnk'
        $shortcut = $shell.CreateShortcut($shortcutPath)
        $shortcut.TargetPath = "$generation\launch.cmd"
        $shortcut.WorkingDirectory = $generation
        $shortcut.IconLocation = "$generation\leditor.exe,0"
        $shortcut.WindowStyle = 7
        $shortcut.Save()
        Set-Content "$base\revision" $revision
        $committed = $true
    } finally { Pop-Location }
    if ((Read-Host 'Remove downloaded/copied source? [Y/n]') -match '^[nN]') {
        $saved = Join-Path ([Environment]::GetFolderPath('UserProfile')) ('leditor-source-' + [guid]::NewGuid().ToString('N').Substring(0,8))
        New-Item -ItemType Directory $saved | Out-Null
        Get-ChildItem -Force $sourceDir | Where-Object { $_.Name -ne 'target' } | Copy-Item -Destination $saved -Recurse
        Write-Host "Source saved: $saved"
    }
    Write-Host "Installed in $base. Open Leditor from the Start menu."
} catch {
    Write-Host "Installation stopped: $($_.Exception.Message)" -ForegroundColor Red
    throw
} finally {
    $env:PATH = $oldPath
    $env:CARGO_HOME = $oldCargo
    $env:CARGO_TARGET_DIR = $oldTarget
    $env:MSYSTEM = $oldMsystem
    $env:CHERE_INVOKING = $oldChere
    $env:CARGO_BUILD_JOBS = $oldJobs
    if ($installLock) {
        $installLock.Dispose()
        Remove-Item -LiteralPath "$base\.install-lock" -Force
    }
    if (!$committed -and $generation -and (Test-Path $generation)) { Remove-Item -LiteralPath $generation -Recurse -Force }
    Remove-TemporaryBuildFiles $work
}
