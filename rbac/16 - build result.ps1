$results = @{}

New-Item -ItemType Directory -Path .\ignore\output -ErrorAction SilentlyContinue | Out-Null
Copy-Item -Path .\ignore\boilerplate.tf -Destination .\ignore\output\boilerplate.tf -ErrorAction SilentlyContinue | Out-Null

# $user_data = Get-Content .\ignore\user_data.tf -Raw
# $result += $user_data
$sections = Get-Content .\ignore\resource_blocks_patched_organized.json | ConvertFrom-Json
foreach ($section in $sections) {
  if ($null -eq $results[$section.File]) {
    $results[$section.File] = ""
  }
  $result = ""
  if ($null -ne $section.Header) {
    $result += "
################
## $($section.Header)
################
"
  }
  foreach ($block in $section.Blocks) {
    $result += "
import {
    id = `"$($block.remote_id)`"
    to = $($block.type).$($block.id)
}
resource `"$($block.type)`" `"$($block.id)`" {`n"
    foreach ($prop in Get-Member -InputObject $block.block.attributes -MemberType NoteProperty) {
        $result += "    " + $prop.Name + " = " + $block.block.attributes.($prop.Name) + "`n"
    }
    $result += "}`n"
  }
  $results[$section.File] += $result
}


foreach ($key in $results.Keys) {
  Set-Content -Value $results[$key] -Path ".\ignore\output\$key"
  terraform fmt ".\ignore\output\$key"
}
