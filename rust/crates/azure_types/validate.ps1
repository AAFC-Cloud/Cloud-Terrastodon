# $x = @($(az policy definition list | ConvertFrom-Json), $(az policy set-definition list | ConvertFrom-Json), $(az policy exemption list | ConvertFrom-Json), $(az policy assignment list | ConvertFrom-Json))

# Define the special characters to match
$specialChars = '#|<|>|\*|%|&|:|\|\?|\+|/'

# Create a hashtable to store the count of each special character
$charCount = @{}

# Initialize the counts to zero for each special character
$specialChars -split '\|' | ForEach-Object { $charCount[$_] = 0 }

# Flatten the array and process each display name
$x
| ForEach-Object { $_.displayName } `
| Where-Object { $null -ne $_ } `
| ForEach-Object {
    $_.ToCharArray() | ForEach-Object {
        if ($_ -match $specialChars) {
            $charCount[$_]++
        }
    }
}

# Display the results
$charCount.GetEnumerator() | Where-Object { $_.Value -gt 0 } | Sort-Object -Property Name | ForEach-Object {
    Write-Output "Character '$($_.Key)' matches: $($_.Value) time(s)"
}
