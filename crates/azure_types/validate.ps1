# Function to count special characters in display names
function Count-SpecialChars {
    param (
        [array]$items,
        [string]$command
    )
    if ($items.Count -eq 0) {
        Write-Output "Results for '$command': (No items found)"
        return;
    }

    $specialChars = '#|<|>|\*|%|&|:|\|\?|\+|/'

    # Create a hashtable to store the count of each special character
    $charCount = @{}

    # Initialize the counts to zero for each special character
    $specialChars -split '\|' | ForEach-Object { $charCount[$_] = 0 }

    # Process each display name
    $items `
    | ForEach-Object { $_.name } `
    | ForEach-Object {
        $name = $_
        $_.ToCharArray() | ForEach-Object {
            $char = $_
            if ($char -match $specialChars) {
                $charCount[$char]++
                Write-Host "Found character $char in $name"
            }
        }
    }

    # Display the results
    Write-Output "Results for '$command':"
    $charCount.GetEnumerator() | Where-Object { $_.Value -gt 0 } | Sort-Object -Property Name | ForEach-Object {
        Write-Output "Character '$($_.Key)' matches: $($_.Value) time(s)"
    }
    Write-Output ""
}

# Get the data from each command
$policyDefinitions = az policy definition list | ConvertFrom-Json
$policySetDefinitions = az policy set-definition list | ConvertFrom-Json
$policyExemptions = az policy exemption list | ConvertFrom-Json
$policyAssignments = az policy assignment list | ConvertFrom-Json

# Count special characters for each command
Count-SpecialChars -items $policyDefinitions -command 'az policy definition list'
Count-SpecialChars -items $policySetDefinitions -command 'az policy set-definition list'
Count-SpecialChars -items $policyExemptions -command 'az policy exemption list'
Count-SpecialChars -items $policyAssignments -command 'az policy assignment list'
