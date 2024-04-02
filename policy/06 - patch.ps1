$generated = Get-Content .\outputs\intermediate\generated-pruned.tf -Raw

# Function to convert JSON string to jsonencode() format
function Convert-ToJsonEncode {
    param (
        [string]$jsonString
    )
    $jsonObject = ConvertFrom-Json "`"$jsonString`"" | ConvertFrom-Json
    return "jsonencode($($jsonObject | ConvertTo-Json -Depth 100))"
}

# Parse and process each block
$resource_blocks = $generated -replace "`r","" -split "`n`n"
$processedBlocks = foreach ($block in $resource_blocks) {
    if ($block -match 'parameters\s*=\s*"(.+)"') {
        $jsonString = $Matches[1]
        $jsonEncode = Convert-ToJsonEncode -jsonString $jsonString
        $block = $block -replace 'parameters\s*=\s*".+"', "parameters = $jsonEncode"
    }
    if ($block -match 'policy_rule\s*=\s*"(.+)"') {
        $jsonString = $Matches[1]
        $jsonEncode = Convert-ToJsonEncode -jsonString $jsonString
        $block = $block -replace 'policy_rule\s*=\s*".+"', "policy_rule = $jsonEncode"
    }
    if ($block -match 'metadata\s*=\s*"(.+)"') {
        $jsonString = $Matches[1]
        $jsonEncode = Convert-ToJsonEncode -jsonString $jsonString
        $block = $block -replace 'metadata\s*=\s*".+"', "metadata = $jsonEncode"
    }
    $block
}

# Join the processed resource_blocks and write to the new file
$processedContent = $processedBlocks -join "`n`n"
Set-Content -Value $processedContent -Path .\outputs\intermediate\generated-pruned-patched.tf
terraform fmt .\outputs\intermediate\generated-pruned-patched.tf