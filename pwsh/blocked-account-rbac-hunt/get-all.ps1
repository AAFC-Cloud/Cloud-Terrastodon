New-Item -ItemType Directory -Path ignore -ErrorAction SilentlyContinue

. .\get-users.ps1
. .\get-groups.ps1
. .\get-role-assignments.ps1

. .\get-groups-with-role-assignments.ps1
. .\get-members-of-groups-with-role-assignments.ps1

. .\get-disabled-presences.ps1