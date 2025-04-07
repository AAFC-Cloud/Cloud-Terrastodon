$data_dir = "$Env:APPDATA\cloud_terrastodon"
# $data_dir = "$Env:APPDATA\cloud_terrastodon\data"
$size_in_gb = (Get-ChildItem $data_dir -force -Recurse -ErrorAction SilentlyContinue| Measure-Object Length -sum).sum / 1Gb
$size_in_gb = [math]::Round($size_in_gb, 2)
Write-Host "Size: $size_in_gb GB"


# ct get-path commands
#old ^^

# new
# ct path list --output json
# ct path get <asd> --output json
# ct path summary <asd> <-- prints size of dir