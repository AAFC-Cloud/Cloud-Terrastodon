$content = Get-Content .\outputs\intermediate\generated.tf
$prune = @"
  administrative_unit_ids    = []
  assignable_to_role         = false
  auto_subscribe_new_members = false
  behaviors                  = []
  external_senders_allowed   = false
  hide_from_address_lists    = false
  hide_from_outlook_clients  = false
  mail_enabled               = false
  onpremises_group_type      = null
  prevent_duplicate_names    = false
  provisioning_options       = []
  theme                      = null
  types                      = []
  visibility                 = null
  writeback_enabled          = false
  condition                              = null
  condition_version                      = null
  delegated_managed_identity_resource_id = null
  description                            = null
  skip_service_principal_aad_check       = null
"@ -replace "`r","" -split "`n"
$new_content = $content | Where-Object {-not ($prune -contains $_) }
Set-Content -Value $new_content -Path .\outputs\intermediate\generated-pruned.tf
