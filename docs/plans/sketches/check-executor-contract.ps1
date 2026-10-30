# Compile the phase 1 proof using Cargo's exact locked workspace artifacts.
# No extra manifest, dependency resolution, credential lookup, or service calls.
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repoRoot
try {
    $artifacts = @(
        cargo test --offline --locked -p cloud_terrastodon_command --test cacheable_work_tests --no-run --message-format=json |
            ForEach-Object { $_ | ConvertFrom-Json } |
            Where-Object { $_.reason -eq 'compiler-artifact' }
    )
    if ($LASTEXITCODE -ne 0) { throw 'Contract dependency compilation failed.' }

    $outputDir = Join-Path $repoRoot 'target/executor-contract-sketch'
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
    $executable = Join-Path $outputDir $(if ($IsWindows) { 'executor-contract.exe' } else { 'executor-contract' })
    $arguments = @('--edition=2024', '--test', (Join-Path $PSScriptRoot 'executor_contract.rs'), '-o', $executable)
    foreach ($crateName in @('cloud_terrastodon_command', 'eyre', 'tokio')) {
        $artifact = @($artifacts | Where-Object { $_.target.name -eq $crateName })
        $library = @($artifact.filenames | Where-Object { $_ -like '*.rlib' } | Select-Object -Unique)
        if ($library.Count -ne 1) { throw "Expected one compiled library for $crateName." }
        $arguments += @('--extern', "$crateName=$($library[0])", '-L', "dependency=$(Split-Path $library[0])")
    }
    # Match the repository's Windows linker selection.
    if ($IsWindows) { $arguments += @('-C', 'linker=rust-lld.exe') }
    & rustc @arguments
    if ($LASTEXITCODE -ne 0) { throw 'Execution contract sketch did not compile.' }
    & $executable
    if ($LASTEXITCODE -ne 0) { throw 'Execution contract sketch checks failed.' }
}
finally {
    Pop-Location
}
