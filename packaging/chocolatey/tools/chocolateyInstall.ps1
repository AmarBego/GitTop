$ErrorActionPreference = 'Stop'

$packageName = 'gittop'

$version = '{{VERSION}}'
$url64 = "https://github.com/AmarBego/GitTop/releases/download/v$version/gittop-$version-setup.exe"
$checksum64 = '{{CHECKSUM}}'
$checksumType64 = 'sha256'

$packageArgs = @{
    packageName    = $packageName
    fileType       = 'exe'
    url64bit       = $url64
    checksum64     = $checksum64
    checksumType64 = $checksumType64
    silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-'
    validExitCodes = @(0)
}

Install-ChocolateyPackage @packageArgs
