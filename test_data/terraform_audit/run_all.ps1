$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition

Get-ChildItem -Path $scriptDir -Directory | ForEach-Object {
    $dirPath = $_.FullName
    Write-Host "Running audit in $dirPath"
    cargo run -- terraform audit $dirPath
}