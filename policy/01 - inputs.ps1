# Define the path to the input file and folder
$inputFolderPath = ".\inputs"
$inputFilePath = "$inputFolderPath\management-group-names.txt"

# Check if the inputs folder exists, create it if not
if (-not (Test-Path -Path $inputFolderPath)) {
    New-Item -Path $inputFolderPath -ItemType "directory"
}

# Check if the input file exists and is not empty
if (-not (Test-Path -Path $inputFilePath) -or [string]::IsNullOrWhiteSpace((Get-Content -Path $inputFilePath -ErrorAction SilentlyContinue))) {
    # Inputs not present, prompt the user
    az account management-group list --no-register -o json `
    | ConvertFrom-Json `
    | Sort-Object -Property displayName `
    | ForEach-Object { $_.displayName + "`t" + $_.id } `
    | fzf --multi --header "Select management groups to import" --bind "ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all" `
    | ForEach-Object { ($_ -split "`t")[1] }
    | ForEach-Object { ($_ -split "/providers/Microsoft.Management/managementGroups/")[1] }
    | Set-Content -Path $inputFilePath
}
