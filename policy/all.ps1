# Define the path to the input file and folder
$inputFolderPath = ".\inputs"
$inputFilePath = "$inputFolderPath\management-group-name.txt"

# Check if the inputs folder exists, create it if not
if (-not (Test-Path -Path $inputFolderPath)) {
    New-Item -Path $inputFolderPath -ItemType "directory"
}

# Check if the input file exists and is not empty
if (-not (Test-Path -Path $inputFilePath) -or [string]::IsNullOrWhiteSpace((Get-Content -Path $inputFilePath -ErrorAction SilentlyContinue))) {
    # Create the file
    New-Item -Path $inputFilePath -ItemType "file" -Force

    # Warn the user and exit
    Write-Warning "Please fill in $inputFilePath (newline separated names) and rerun the script."
    exit
}


# Check if outputs directory exists
$outputFolderPath = "outputs"
if (Test-Path -Path $outputFolderPath) {
    # Ask user if they want to delete the outputs directory
    Write-Host -ForegroundColor DarkYellow -NoNewline "The 'outputs' directory already exists. Do you want to delete it? (y/N) "
    $userInput = Read-Host

    # Check user response and delete if confirmed
    if ($userInput -eq 'Y' -or $userInput -eq 'y') {
        Remove-Item -Path $outputFolderPath -Recurse -Force
        Write-Host "'outputs' directory deleted."
    }
}

Write-Host -ForegroundColor Yellow "Calling `"01 - get assignments.ps1`""
. ".\01 - get assignments.ps1"
Write-Host -ForegroundColor Yellow "Calling `"02 - get definitions.ps1`""
. ".\02 - get definitions.ps1"
Write-Host -ForegroundColor Yellow "Calling `"03 - build imports.ps1`""
. ".\03 - build imports.ps1"
Write-Host -ForegroundColor Yellow "Calling `"04 - terraform generate.ps1`""
. ".\04 - terraform generate.ps1"
Write-Host -ForegroundColor Yellow "Calling `"05 - prune defaults.ps1`""
. ".\05 - prune defaults.ps1"
Write-Host -ForegroundColor Yellow "Calling `"06 - patch.ps1`""
. ".\06 - patch.ps1"