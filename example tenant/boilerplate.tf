terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = ">=3.81.0"
    }
    azuread = {
      source  = "hashicorp/azuread"
      version = ">=2.46.0"
    }
    random = {
      source = "hashicorp/random"
      version = ">=3.6.0"
    }
  }
}

provider "azurerm" {
  features {}
  skip_provider_registration = true
}

data "azurerm_client_config" "main" {}
