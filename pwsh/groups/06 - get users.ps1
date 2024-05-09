$content = Get-Content .\outputs\intermediate\generated-pruned.tf 
    | Where-Object { $_ -like "*members*=*" -or $_ -like "*owners*=*" }
$userObjectIds = $content 
    | ForEach-Object { ConvertFrom-Json ($_ -split "=")[1] } 
    | Get-Unique

# Function to batch the requests
function Invoke-BatchProcess($ids, $batchSize) {
    $batchedResults = @()
    for ($i = 0; $i -lt $ids.Count; $i += $batchSize) {
        $j = $i + $batchSize - 1
        Write-Progress -PercentComplete ($i/$ids.Count*100) -Activity "Batch $i-$j"
        $currentBatch = $ids[$i..$j]
        $body = @{ ids = $currentBatch } | ConvertTo-Json -Compress
        $body = $body -replace '"', '\"'
        $results = az rest `
            --method POST `
            --url 'https://graph.microsoft.com/v1.0/directoryObjects/getByIds' `
            --headers 'Content-Type=application/json' `
            --body $body
        $batchedResults += $results | ConvertFrom-Json
    }
    Write-Progress -Completed
    return $batchedResults
}

# Process in batches of 20
$allResults = Invoke-BatchProcess -ids $userObjectIds -batchSize 20


# Combine and save the results
$users = $allResults | ForEach-Object { $_.value }
Set-Content -Value ($users | ConvertTo-Json) -Path ".\outputs\intermediate\users.json"
