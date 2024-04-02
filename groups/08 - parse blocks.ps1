$generated = Get-Content .\outputs\intermediate\generated-pruned.tf -Raw

# skip the first block which is just comments
$blocks = $generated -replace "`r","" -split "`n`n" | Select-Object -Skip 1

$results = @()
foreach ($block in $blocks) {
    $lines = $block.Trim() -split "`n"
    $null, $type, $null, $id, $null = $lines[1] -split "`""
    $body = $lines[2..($lines.Count-2)]
    $properties = @{}
    foreach ($line in $body) {
        $key, $value = $line -split "=",2
        $properties[$key.Trim()] = $value.Trim()
    }
    
    $results += @{
        type = $type
        id = $id
        properties = $properties
    }
}
Set-Content -Value ($results | ConvertTo-Json -Depth 5) -Path .\outputs\intermediate\group_blocks.json
