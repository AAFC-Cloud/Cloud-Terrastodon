Push-Location .\outputs\intermediate
try {
    Remove-Item generated.tf -ErrorAction SilentlyContinue
    Remove-Item generated-pruned.tf -ErrorAction SilentlyContinue
    Remove-Item generated-pruned-patched.tf -ErrorAction SilentlyContinue
    Set-Content boilerplate.tf @"
provider "azurerm" {
    features {}
    skip_provider_registration = true
}
"@
    terraform init
    terraform plan -generate-config-out="generated.tf"
} finally {
    Pop-Location
}