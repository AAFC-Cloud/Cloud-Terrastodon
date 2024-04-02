$groups = az ad group list
Set-Content -Path outputs\intermediate\groups.json -Value $groups
