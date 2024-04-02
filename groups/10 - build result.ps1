$result = ""

$user_data = Get-Content .\outputs\intermediate\user_data.tf -Raw
$result += $user_data

$group_lookup = Get-Content .\outputs\intermediate\lookup.json `
    | ConvertFrom-Json -AsHashtable

$blocks = Get-Content .\outputs\intermediate\group_blocks_patched.json `
    | ConvertFrom-Json

foreach ($block in $blocks) {
    $group = $group_lookup[($block.properties.display_name -split "`"")[1]]
    $result += "
################
## $($block.id)
################
import {
  id = `"$($group.id)`"
  to = $($block.type).$($block.id)
}
resource `"$($block.type)`" `"$($block.id)`" {`n"

    foreach ($prop in Get-Member -InputObject $block.properties -MemberType NoteProperty) {
        $result += "    " + $prop.Name + " = " + $block.properties.($prop.Name) + "`n"
    }
    $result += "}`n"
}

Set-Content -Value $result -Path .\outputs\main.tf
terraform fmt .\outputs\main.tf