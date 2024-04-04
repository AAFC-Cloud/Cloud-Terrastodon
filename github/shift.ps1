param (
    [Parameter(Mandatory=$true)]
    [int]$Shift
)

function Update-ScriptNumber {
    param (
        [string]$FileName,
        [int]$Shift
    )

    # Extract the numeric prefix and rest of the filename
    if ($FileName -match '^(\d+) - (.*)$') {
        $number = [int]$Matches[1]
        $rest = $Matches[2]
        
        # Calculate new number with leading zeros
        $newNumber = '{0:D2}' -f ($number + $Shift)
        
        # Check for negative or zero numbering
        if ($newNumber -le 0) {
            Write-Warning "Skipping $FileName as shift results in non-positive numbering."
            return
        }
        
        # Form new file name
        $newFileName = "$newNumber - $rest"
        
        # Rename the file
        Rename-Item -Path $FileName -NewName $newFileName
        Write-Host "Renamed $FileName to $newFileName"
    }
    else {
        Write-Warning "File $FileName does not match expected pattern."
    }
}

# Gather script files
$scriptFiles = Get-ChildItem -Path . -Filter *.ps1 | Where-Object { $_.Name -match "^\d\d -.*"} | Sort-Object Name

# Convert script files to a simple list for fzf
$scriptNames = $scriptFiles | ForEach-Object { $_.Name }

# Use fzf to allow the user to pick scripts. Adjust the path to fzf if necessary.
$selectedScripts = $scriptNames | fzf --multi --cycle --layout=reverse --header="Select scripts to shift by ${Shift}:" | Out-String -Stream

# Filter $scriptFiles to match only those that were selected and perform the shift
$scriptFiles | Where-Object { $selectedScripts -contains $_.Name } | ForEach-Object {
    Update-ScriptNumber -FileName $_.Name -Shift $Shift
}
