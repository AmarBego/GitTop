$ErrorActionPreference = 'Stop'

$packageName = 'gittop'

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
