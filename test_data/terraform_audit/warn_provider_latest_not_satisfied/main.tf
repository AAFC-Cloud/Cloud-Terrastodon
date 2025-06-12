terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "<4.0.0"
    }
  }
}

resource "azurerm_storage_account" "bruh" {
  
}