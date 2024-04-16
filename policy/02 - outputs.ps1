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
