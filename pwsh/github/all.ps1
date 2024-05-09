param (
    [switch]$Pick
)

# Gather script files, excluding all.ps1
$scripts = Get-ChildItem -Path . -Filter *.ps1 `
| Where-Object { $_.Name -match "\d\d -.*"} | Sort-Object Name

if ($Pick) {
    # Convert script files to a simple list for fzf
    $script_names = $scripts | ForEach-Object { $_.Name }

    # Use fzf to allow the user to pick scripts. Make sure to adjust the path to fzf if necessary
    $chosen = $script_names `
    | fzf --multi --cycle --layout=reverse | Out-String -Stream
    
    # Filter $scripts to match only those that were selected
    $scripts = $scripts `
    | Where-Object { $chosen -contains $_.Name }
}

# Execute the selected or all scripts
foreach ($script in $scripts) {
    Write-Host "Executing $($script.Name)"
    & $script.FullName
    if ($? -eq $false) {
        Write-Warning "Script $($script.Name) failed, abandoning remaining steps"
        exit 1
    }
}
