$blocks = Get-Content .\outputs\intermediate\group_blocks.json | ConvertFrom-Json
$users = Get-Content .\outputs\intermediate\users.json `
| ConvertFrom-Json `
| Group-Object -Property "id" -AsHashTable

foreach ($block in $blocks) {
  $block.properties = Select-Object -Property * -ExcludeProperty mail_nickname -InputObject $block.properties
  if ($block.properties.description -eq "null") {
    $block.properties = Select-Object -Property * -ExcludeProperty description -InputObject $block.properties
  }
  if ($block.properties.owners -eq "[]") {
    $block.properties = Select-Object -Property * -ExcludeProperty owners -InputObject $block.properties
    Add-Member -MemberType NoteProperty -Name "#owners" -Value "[]" -InputObject $block.properties
  } else {
    $owners = $block.properties.owners `
    | ConvertFrom-Json
    | ForEach-Object { $users[$_][0] }
    | ForEach-Object { $_.userPrincipalName }
    | ForEach-Object { "local.user_id_by_mail[`"$_`"]," }
    $block.properties.owners = "[
  $($owners -join "`n    ")
]"    
  }
  if ($block.properties.members -eq "[]") {
    $block.properties = Select-Object -Property * -ExcludeProperty members -InputObject $block.properties
    Add-Member -MemberType NoteProperty -Name "#members" -Value "[]" -InputObject $block.properties
  } else {
    $members = $block.properties.members `
    | ConvertFrom-Json
    | ForEach-Object { $users[$_][0] }
    | ForEach-Object { $_.userPrincipalName }
    | ForEach-Object { "local.user_id_by_mail[`"$_`"]," }
    $block.properties.members = "[
$($members -join "`n    ")
]"
  }
}
Set-Content -Value ($blocks | ConvertTo-Json -Depth 5) -Path .\outputs\intermediate\group_blocks_patched.json