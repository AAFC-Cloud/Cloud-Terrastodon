Push-Location .\ignore\
try {
    Remove-Item generated.tf -ErrorAction SilentlyContinue
    Remove-Item generated-pruned.tf -ErrorAction SilentlyContinue
    Remove-Item main.tf -ErrorAction SilentlyContinue
    Remove-Item .terraform.lock.hcl -ErrorAction SilentlyContinue
    # terraform {
    #     required_providers {
    #         azurerm = {
    #             source  = "hashicorp/azurerm"
    #             version = "3.84.0" # https://github.com/hmcts/rd-shared-infrastructure/pull/233
    #         }
    #         azuread = {
    #             source  = "hashicorp/azuread"
    #             #version = ">=2.31.0"
    #         }
    #     }
    #     }
    Set-Content boilerplate.tf @"
provider "azurerm" {
    features {}
    skip_provider_registration = true
}
"@
    terraform init
    if ($? -eq $false) {
        Write-Warning "Failed to initialize Terraform"
        exit 1
    }

    Write-Host "Performing Terraform generation from import blocks..." -ForegroundColor Cyan
    terraform plan -generate-config-out="generated.tf" 2>&1 > terraform_plan_generate_config_out.log
    # if ($? -eq $false) {
    #     Get-Content terraform_plan_generate_config_out.log | Out-Host
    #     exit 1
    # }
    $errors = @(
        "Error: building client:"
        "Error: building account:"
        "Error: Invalid provider configuration"
        "Error: Cannot import non-existent remote object.*" 
    )
    foreach ($e in $errors) {
        rg "$e" --after-context 8 terraform_plan_generate_config_out.log
        if ($? -eq $true) {
            Write-Warning "Found errors in `".\ignore\terraform_plan_generate_config_out.log`". Aborting."
            exit 1
        }    
    }
} finally {
    Pop-Location
}