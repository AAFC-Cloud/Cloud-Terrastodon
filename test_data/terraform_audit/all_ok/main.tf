terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = ">=4.18.0"
    }
  }
}

resource "azurerm_storage_account" "bruh" {
  
}