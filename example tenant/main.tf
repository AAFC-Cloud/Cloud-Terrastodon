locals {
  users = {
    for user in [
      {
        name  = "Atla Palani"
        email = "atla.palani@terrastodon.onmicrosoft.com"
        groups = [
          "Colour identity includes green",
          "Colour identity includes red",
          "Colour identity includes white",
          "Egg Enthusiasts",
          "Commanders",
        ]
      },
      {
        name  = "Phylath"
        email = "phylath@terrastodon.onmicrosoft.com"
        groups = [
          "Colour identity includes green",
          "Colour identity includes red",
          "Land Enthusiasts",
          "Commanders",
        ]
      },
      {
        name  = "Nesting Dragon"
        email = "nesting.dragon@terrastodon.onmicrosoft.com"
        groups = [
          "Colour identity includes red",
          "Egg Enthusiasts"
        ]
      }
    ] : user.name => user
  }
  groups = toset(flatten([for _, user in local.users : user.groups]))
}

resource "azuread_group" "main" {
  for_each         = local.groups
  security_enabled = true
  mail_enabled     = false
  display_name     = each.value
  owners           = [data.azurerm_client_config.main.object_id]
  members          = [for k, v in local.users : azuread_user.main[k].object_id if contains(v.groups, each.value)]
}


resource "random_password" "main" {
  for_each = local.users
  length   = 64
}

resource "azuread_user" "main" {
  for_each            = local.users
  display_name        = each.value.name
  user_principal_name = each.value.email
  password            = random_password.main[each.key].result
}
