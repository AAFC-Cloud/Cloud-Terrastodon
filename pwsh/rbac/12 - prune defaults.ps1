$content = Get-Content .\ignore\generated.tf
# metadata             =*
$prune = @"
# __generated__ by Terraform*
# Please review these resources and move them into your main configuration files.
  condition                              = null
  condition_version                      = null
  delegated_managed_identity_resource_id = null
  role_definition_id                     =*
  role_definition_id                     =*
  skip_service_principal_aad_check       = null
  administrative_unit_ids    = []
  assignable_to_role         = false
  auto_subscribe_new_members = false
  behaviors                  = []
  external_senders_allowed   = false
  hide_from_address_lists    = false
  hide_from_outlook_clients  = false
  mail_enabled               = false
  owners                     = []
  members                    = []
  mail_nickname              =*
  description                = null
  onpremises_group_type      = null
  prevent_duplicate_names    = false
  provisioning_options       = []
  theme                      = null
  types                      = []
  visibility                 = null
  writeback_enabled          = false
"@ -replace "`r","" -split "`n"
$prunePatterns = $prune | Where-Object { $_.Contains("*") }
$new_content = $content | Where-Object {
    $line = $_
    if ($prune -contains $line) {
        return $false
    }
    $matched = $prunePatterns | Where-Object { $line -like $_ }
    return $matched.Count -eq 0
}
Set-Content -Value $new_content -Path .\ignore\generated-pruned.tf


$contentLength = $content.Length
$newContentLength = $new_content.Length

# Calculate the reduction in size
$reduction = $contentLength - $newContentLength

# Calculate the percentage reduction and round it to the nearest whole number
$percentageReduction = [Math]::Round(($reduction / $contentLength) * 100)

# Display the result using Write-Host
Write-Host "The content was reduced by $percentageReduction%."

