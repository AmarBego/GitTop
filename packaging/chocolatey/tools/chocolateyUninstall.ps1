$ErrorActionPreference = 'Stop'

# MSI uninstallation is handled automatically by Windows Installer
# User settings are stored in $env:APPDATA\GitTop
# To remove settings, delete that folder manually or run:
#   Remove-Item "$env:APPDATA\GitTop" -Recurse -Force
