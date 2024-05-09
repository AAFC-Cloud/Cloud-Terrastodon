Push-Location .\outputs\intermediate
try {
    Remove-Item generated.tf -ErrorAction SilentlyContinue
    Remove-Item generated-pruned.tf -ErrorAction SilentlyContinue
    terraform init
    terraform plan -generate-config-out="generated.tf"
} finally {
    Pop-Location
}