# Define the path to the input file and folder
$inputFolderPath = ".\inputs"
$inputFilePath = "$inputFolderPath\group_names.txt"

# Check if the inputs folder exists, create it if not
if (-not (Test-Path -Path $inputFolderPath)) {
    New-Item -Path $inputFolderPath -ItemType "directory"
}

# Check if the input file exists and is not empty
if (-not (Test-Path -Path $inputFilePath) -or [string]::IsNullOrWhiteSpace((Get-Content -Path $inputFilePath -ErrorAction SilentlyContinue))) {
    # Inputs not present, prompt the user
    az ad group list --query [].displayName -o tsv `
    | Sort-Object `
    | fzf --multi --header "Select groups to import" --bind "ctrl-a:select-all,ctrl-d:deselect-all,ctrl-t:toggle-all" `
    | Set-Content -Path $inputFilePath
}

New-Item -ItemType Directory -Path outputs\intermediate -Force

. ".\01 - get groups.ps1"
. ".\02 - build lookup.ps1"
. ".\03 - create_tf.ps1"
. ".\04 - terraform gen.ps1"
. ".\05 - prune defaults.ps1"
. ".\06 - get users.ps1"
. ".\07 - build user data.ps1"
. ".\08 - parse blocks.ps1"
. ".\09 - patch blocks.ps1"
. ".\10 - build result.ps1"