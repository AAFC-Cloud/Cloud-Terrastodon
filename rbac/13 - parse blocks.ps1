function ConvertFrom-TerraformBlock {
    param (
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$BlockContent
    )

    # Write-Host -ForegroundColor DarkYellow "Received BlockContent`n```````n$BlockContent`n``````"

    $attributes = @{}
    $blocks = @{}

    # Splitting the block content into lines and handling Windows-style line endings
    $lines = ($BlockContent -replace "`r","").Trim().Split("`n")

    $currentBlockName = $null
    $currentBlockIndentation = $null
    $currentBlockLines = @()

    foreach ($line in $lines) {
        # Write-Host -ForegroundColor DarkMagenta "Processing `"$line`""
        if ($currentBlockName -and $line -match "^$currentBlockIndentation\}") { # End of a nested block
            if (-not $blocks.ContainsKey($currentBlockName)) {
                $blocks[$currentBlockName] = @()
            }
            # Write-Host -ForegroundColor "Cyan" "Got end: $line"
            # $blocks[$currentBlockName] += @{ 'attributes' = $blockAttributes; 'blocks' = $nestedBlocks }
            $blocks[$currentBlockName] += ConvertFrom-TerraformBlock -BlockContent ($currentBlockLines -join "`n")
            $currentBlockName = $null
            $currentBlockLines = @()
            continue
        }
        elseif ($currentBlockName) {
            # Write-Host -ForegroundColor Cyan "Appending to current block"
            $currentBlockLines += $line
        }
        elseif ($line -match "^(\s*)(\w+)\s+\{") { # Start of a nested block
            $currentBlockName = $Matches[2]
            $currentBlockIndentation = $Matches[1]
            # Write-Host -ForegroundColor Cyan "Starting block `"$currentBlockIndentation$currentBlockName`""
            continue
        }
        elseif ($line -match "^\s*([\w_]+)\s*=\s*(.+)") { # Attribute line
            $key = $Matches[1]
            $value = $Matches[2]
            # Write-Host -ForegroundColor Cyan "Found attribute $key=$value"
            $attributes[$key] = $value
        }
    }

    return @{
        attributes = $attributes
        blocks = $blocks
    }
}


$imports = @(
    $(Get-Content .\ignore\group_imports.tf -Raw)
    $(Get-Content .\ignore\pim_imports.tf -Raw)
    $(Get-Content .\ignore\role_assignment_imports.tf -Raw)
) -join "`n"
$import_blocks = $imports.Trim() -replace "`r","" -split "}`n+"
$id_lookup = $import_blocks `
    | Foreach-Object {
        $id, $to = ($_ -split "`"|= |`n")[3,6]
        $type,$resource = ($to -split "\.")
        return [PSCustomObject]@{
            Id=$id;
            To=$to;
            Type=$type;
            Resource=$resource
        }
    } `
    | Foreach-Object -Begin { $lookup = @{} } -Process {
        if ($null -eq $lookup[$_.Type]) {
            $lookup[$_.Type] = @{}
        }
        $lookup[$_.Type][$_.Resource] = $_
    } -End { $lookup }
$id_lookup.GetEnumerator() `
    | ForEach-Object { [PSCustomObject]@{
        resource_type = $_.Name
        cache_size = $_.Value.Count
    }} `
    | Format-Table


$generated = Get-Content .\ignore\generated-pruned.tf -Raw
$resource_blocks = $generated.Trim() -replace "`r","" -split "`n`n"
$results = @()
foreach ($block in $resource_blocks) {
    $lines = $block.Trim() -split "`n"
    $null, $type, $null, $id, $null = $lines[0] -split "`""

    $body = $lines[1..($lines.Count-2)]
    $block = $body -join "`n" | ConvertFrom-TerraformBlock
    
    $remote_id = $id_lookup[$type][$id].Id
    
    $results += @{
        type = $type
        id = $id
        remote_id = $remote_id
        block = $block
    }
}
Set-Content -Value ($results | ConvertTo-Json -Depth 100) -Path .\ignore\resource_blocks.json
Write-Host -ForegroundColor Green "Wrote resource_blocks.json with $($results.Count) entries"