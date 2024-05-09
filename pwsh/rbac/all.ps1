param (
    [switch]$Pick
)

Write-Host "Creating ignore directory if not exists"
New-Item -ItemType Directory -Path ignore -ErrorAction SilentlyContinue | Out-Null

# Gather script files, excluding all.ps1
$scriptFiles = Get-ChildItem -Path . -Filter *.ps1 | Where-Object { $_.Name -match "\d\d -.*"} | Sort-Object Name
    

if ($Pick) {
    # Convert script files to a simple list for fzf
    $scriptNames = $scriptFiles | ForEach-Object { $_.Name }
    # Use fzf to allow the user to pick scripts. Make sure to adjust the path to fzf if necessary
    $selectedScripts = $scriptNames | fzf --multi --cycle --layout=reverse | Out-String -Stream
    # Filter $scriptFiles to match only those that were selected
    $scriptFiles = $scriptFiles | Where-Object { $selectedScripts -contains $_.Name }
}

# Execute the selected or all scripts
foreach ($script in $scriptFiles) {
    Write-Host "Executing $($script.Name)"
    . $script.FullName
    if ($? -eq $false) {
        Write-Warning "Script $($script.Name) failed, abandoning remaining steps"
        exit 1
    }
}
