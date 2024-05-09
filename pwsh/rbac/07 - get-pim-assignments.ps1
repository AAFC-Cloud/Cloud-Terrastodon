if (Test-Path .\ignore\pim_assignments.json) {
    $pim_assignments = Get-Content .\ignore\pim_assignments.json | ConvertFrom-Json
    Write-Host "Loaded $($pim_assignments.Length) lines from pim_assignments.json cache"
} else {
    if ($null -eq $management_groups) {
        Write-Warning "Variable missing. Please run previous steps."
        exit 1
    }
    $pim_assignments = $management_groups `
        | ForEach-Object -ThrottleLimit 20 -Parallel {
            $mg = $_
            Write-Host "Fetching PIM assignments from $($mg.displayName)"
            $url = "`"https://management.azure.com$($mg.id)/providers/Microsoft.Authorization/roleEligibilityScheduleInstances?api-version=2020-10-01`""
            az rest --method get --url $url | ConvertFrom-Json | Select-Object -ExpandProperty value
        }
    Set-Content -Path .\ignore\pim_assignments.json -Value $($pim_assignments | ConvertTo-Json -Depth 100)
    Write-Host "Fetched $($pim_assignments.Length) lines into pim_assignments.json"
}
Write-Host "Found $($pim_assignments.Count) pim assignments!" -ForegroundColor Green
