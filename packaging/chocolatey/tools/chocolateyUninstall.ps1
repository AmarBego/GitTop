$ErrorActionPreference = 'Stop'

$packageName = 'gittop'

# Find the Inno Setup uninstaller in LocalAppData
$uninstallPath = Join-Path $env:LOCALAPPDATA 'GitTop\unins000.exe'

if (Test-Path $uninstallPath) {
    $packageArgs = @{
        packageName    = $packageName
        fileType       = 'exe'
        file           = $uninstallPath
        silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART'
        validExitCodes = @(0)
    }

    Uninstall-ChocolateyPackage @packageArgs
}
else {
    Write-Warning "Uninstaller not found at $uninstallPath - GitTop may have been removed manually"
}

# Note: User settings are stored in $env:APPDATA\GitTop
# These are preserved by default. To remove settings, delete that folder manually.
