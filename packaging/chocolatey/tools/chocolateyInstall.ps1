$ErrorActionPreference = 'Stop'

$packageName = 'gittop'
$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"

$version = '{{VERSION}}'
$url64 = "https://github.com/AmarBego/GitTop/releases/download/v$version/gittop-$version.msi"
$checksum64 = '{{CHECKSUM}}'
$checksumType64 = 'sha256'

$packageArgs = @{
    packageName    = $packageName
    fileType       = 'msi'
    url64bit       = $url64
    checksum64     = $checksum64
    checksumType64 = $checksumType64
    silentArgs     = '/quiet /norestart'
    validExitCodes = @(0, 3010)
}

Install-ChocolateyPackage @packageArgs
