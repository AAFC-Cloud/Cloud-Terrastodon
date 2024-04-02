$groups_with_role_assignments = $role_assignments `
    | ForEach-Object { $groupz[$_.principalId]} `
    | Where-Object { $null -ne $_ }
    | Sort-Object -Unique -Property id
Write-Host "Found $($groups_with_role_assignments.Count) groups with role assignments!" -ForegroundColor Green