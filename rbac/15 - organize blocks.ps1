$resource_blocks = Get-Content .\ignore\resource_blocks_patched.json `
    | ConvertFrom-Json

$group_blocks = $resource_blocks `
    | Where-Object { $_.type -eq "azuread_group" } `
    | Sort-Object { $_.id }
$role_assignment_blocks = $resource_blocks `
    | Where-Object { $_.type -eq "azurerm_role_assignment" } `
    | Sort-Object { $_.id }
$pim_blocks = $resource_blocks `
    | Where-Object { $_.type -eq "azurerm_pim_eligible_role_assignment" } `
    | Sort-Object { $_.id }

$visited_role_assignments = New-Object 'System.Collections.Generic.HashSet[string]'

$sections = @()
foreach ($group_block in $group_blocks) {
    $section = [PSCustomObject]@{
        File = "group_role_assignments.tf"
        Header = $group_block.block.attributes.display_name | ConvertFrom-Json
        Blocks = @()
    }
    $section.Blocks += $group_block
    $role_assignment_blocks `
    | Where-Object { $_.block.attributes.principal_type -eq '"Group"' } `
    | Where-Object { $_.block.attributes.principal_id -eq "azuread_group.$($group_block.id).object_id" } `
    | ForEach-Object {
        $section.Blocks += $_
        $visited_role_assignments.Add($_.id) | Out-Null
    } 
    
    $sections += $section
}

foreach ($ra in $role_assignment_blocks) {
    if ($visited_role_assignments.Contains($ra.id)) {
        continue
    }
    $file = switch($ra.block.attributes.principal_type) {
        '"Group"' { "group_role_assignments.tf" }
        '"ServicePrincipal"' { "service_principal_role_assignments.tf" }
        '"User"' { "user_role_assignments.tf" }
    }
    $section = [PSCustomObject]@{
        File = $file
        Header = $null #"$($ra.block.attributes.principal_type) $($ra.block.attributes.role_definition_name)"  
        Blocks = @()
    }
    $section.Blocks += $ra
    $sections += $section
}

foreach ($pim_block in $pim_blocks) {
    $section = [PSCustomObject]@{
        File = "pim_assignments.tf"
        Header = $null
        Blocks = @()
    }
    $section.Blocks += $pim_block
    $sections += $section
}


if ($sections.Blocks.Count -ne $resource_blocks.Count) {
    Write-Warning "Block counts mismatch! Expected $($resource_blocks.Count), got $($sections.Blocks.Count)"
    exit 1
}

$sections `
    | ConvertTo-Json -Depth 100 `
    | Set-Content -Path ".\ignore\resource_blocks_patched_organized.json"