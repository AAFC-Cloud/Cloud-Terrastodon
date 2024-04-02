$resource_blocks = Get-Content .\ignore\resource_blocks.json | ConvertFrom-Json

function deref_object_id {
  param (
    [Parameter(Mandatory = $true)]
    [string]$id,

    [switch]$comma
  )

  # Determine comma usage based on switch
  $c = if ($comma) { "," } else { "" }

  if ($userz.ContainsKey($id)) {
    return "local.user_id_by_mail[`"$($userz[$id].userPrincipalName)`"]$c"
  }
  elseif ($groupz.ContainsKey($id)) {
    $group = $groupz[$id]
    $rn = $group.ResourceName
    if ([string]::IsNullOrWhitespace($rn)) {
      return "`"$id`"$c # GROUP NOT MANAGED BY THIS PROJECT - $($group.DisplayName)"
    }
    else {
      return "azuread_group.$rn.object_id$c"
    }
  } 
  elseif ($service_principalz.ContainsKey($id)) {
    $sp = $service_principalz[$principal_id]
    $name = $sp.displayName
    if (-not [string]::IsNullOrWhitespace($name)) {
      return "`"$principal_id`" # NOT MANAGED BY THIS PROJECT - $($name)"
    }
  }
  return "`"$id`"$c # UNKNOWN OBJECT!"
}


foreach ($block in $resource_blocks) {
  if ($block.type -eq "azuread_group") {
    if ($null -eq $block.block.attributes.owners) {
      Add-Member -MemberType NoteProperty -Name "#owners" -Value "[]" -InputObject $block.block.attributes
    } else {
      $owners = $block.block.attributes.owners `
      | ConvertFrom-Json
      | ForEach-Object { deref_object_id -id $_ -comma }
      $block.block.attributes.owners = "[
    $($owners -join "`n    ")
  ]"    
    }
    if ($null -eq $block.block.attributes.members) {
      Add-Member -MemberType NoteProperty -Name "#members" -Value "[]" -InputObject $block.block.attributes
    } else {
      $members = $block.block.attributes.members `
      | ConvertFrom-Json
      | ForEach-Object { deref_object_id -id $_ -comma }
      $block.block.attributes.members = "[
  $($members -join "`n    ")
  ]"
    }
  } elseif ($block.type -eq "azurerm_role_assignment" ) {
    $principal_id = $block.block.attributes.principal_id | ConvertFrom-Json
    $block.block.attributes.principal_id = $(deref_object_id -id $principal_id)
  }
}
Set-Content -Value ($resource_blocks | ConvertTo-Json -Depth 100) -Path .\ignore\resource_blocks_patched.json