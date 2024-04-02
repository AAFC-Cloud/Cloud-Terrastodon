$pattern = "user_id_by_mail\[`"(.*?)`"\]"
$used_emails = New-Object System.Collections.Generic.HashSet[string]

$tf_files = Get-ChildItem .\ignore\output
foreach ($file in $tf_files) {
    $content = Get-Content -Path $file
    $found = [regex]::Matches($content, $pattern)
    foreach ($match in $found) {
        $used_emails.Add($match.Groups[1].Value) | Out-Null
    }
}

$template = 'data "azuread_users" "users" {
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

$user_list = $used_emails `
| ForEach-Object { "`"$_`"," }
$result = $template -replace "%USER_LIST%", ($user_list -join "`n    ")
Set-Content -Value $result -Path .\ignore\output\user_lookup.tf