# List projects
$projects = az devops project list `
| ConvertFrom-Json `
| Select-Object -ExpandProperty value

Write-Host "Found $($projects.Count) projects!"

# Get pipelines
$runs = @{}
$found = $projects | ForEach-Object -ThrottleLimit 20 -Parallel {
    $project = $_
    Write-Host "Fetching pipelines for $($project.name)"
    $runs = az pipelines runs list --project $project.name | ConvertFrom-Json
    return [PSCustomObject]@{
        ProjectName = $project.name;
        Runs = $runs;
    }
}
foreach ($entry in $found) {
    $runs[$entry.ProjectName] = $entry.Runs
}

$outfile = "report.md"
foreach ($project in $projects) {
    Add-Content -Path $outfile -Value "# $($project.name)"
    if ($runs[$project.name].Count -eq 0) {
        Add-Content -Path $outfile -Value "No pipelines."
        continue
    }
    $runz = $runs[$project.name] | Group-Object -Property { $_.definition.name }
    foreach ($group in $runz) {
        $latest = $group.Group.finishTime | Measure-Object -Maximum | Select-Object -ExpandProperty Maximum
        Add-Content -Path $outfile -Value "- $($group.Name) - $($group.Count) runs - latest $latest"
    }
}