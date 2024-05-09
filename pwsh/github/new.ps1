$numbers = Get-ChildItem `
| Where-Object { $_.Name -match '^(\d+) - .*\.ps1$' } `
| ForEach-Object { [int]$Matches[1] } `
| Sort-Object -Unique

$targetNumber = 1 # Start from 1
foreach ($number in $numbers) {
    if ($number -eq $targetNumber) {
        $targetNumber++ # Increment to find the next potential missing number
    } else {
        break # Found the gap
    }
}

$name = Read-Host "Name without number or extension"
$file = "{0:D2} - {1}.ps1" -f $targetNumber, $name
$null = code $file &