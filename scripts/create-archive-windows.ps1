param(
    [Parameter(Mandatory=$true)]
    [string]$Target,

    [Parameter(Mandatory=$true)]
    [string]$Version,

    [Parameter(Mandatory=$true)]
    [string]$Workspace
)

$ErrorActionPreference = "Stop"

$releaseDir = "target/$Target/release"
$output = "ar7json-$Version-$Target.zip"

Set-Location $releaseDir
Compress-Archive -Path "ar7json.exe","$Workspace/completions/ar7json.powershell","$Workspace/man/ar7json.1" -DestinationPath "../../../$output"
