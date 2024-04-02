$content = 'data "azuread_users" "users" {
  user_principal_names = [
    %USER_LIST%
  ]
}
locals {
  user_id_by_mail = {
    for x in data.azuread_users.users.users :
    x.user_principal_name => x.object_id
  }
}'

$users = Get-Content .\outputs\intermediate\users.json | ConvertFrom-Json
$user_list = $users `
    | ForEach-Object { "`"$($_.userPrincipalName)`"," } `
    | Sort-Object -Unique

$result = $content -replace "%USER_LIST%",($user_list -join "`n    ")
Set-Content -Value $result -Path .\outputs\intermediate\user_data.tf