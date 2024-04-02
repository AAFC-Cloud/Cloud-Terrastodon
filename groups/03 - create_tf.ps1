$lookup = Get-Content .\outputs\intermediate\lookup.json | ConvertFrom-Json -AsHashtable

# We want to build an imports.tf file
$entry_template = @"
import {
    id = "%ID%"
    to = azuread_group.%NAME%
}

"@
$content = ""

function sanitize($name) {
    $name -replace "-| ","_"
}
foreach ($group in $lookup.Values) {
    $content += $entry_template -replace "%ID%",$group.id -replace "%NAME%",$(sanitize $group.displayName)
}
Set-Content ".\outputs\intermediate\imports.tf" $content