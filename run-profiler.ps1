param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$QueryArgs
)

if (-not $QueryArgs -or $QueryArgs.Count -eq 0) {
    $QueryArgs = @("az", "resource", "list")
}

$profiler = Get-Command teamy-profiler -ErrorAction SilentlyContinue
if (-not $profiler) {
    throw "teamy-profiler not found in PATH"
}

$profiler.Source run cargo `
    --project $PSScriptRoot `
    --bin cloud_terrastodon `
    --profile release `
    --feature tracy-alloc `
    -- @QueryArgs

return $LASTEXITCODE
