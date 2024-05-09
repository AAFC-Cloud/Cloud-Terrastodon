$out_file = ".\ignore\terraform\boilerplate.tf"

$content = '
terraform {
  required_providers {
    github = {
      source  = "integrations/github"
      version = "6.2.1"
    }
  }
}
'.Trim()

Set-Content -Path $out_file -Value $content