# CLI and egui parity

Every egui data page and domain action needs a discoverable Cloud Terrastodon
CLI equivalent. Add both entry points together, backed by the same request
objects and domain workflows. Review account selection, tenant and service
scope, server-side filters, refresh behavior, and write verification when
checking parity.

The CLI can return structured data that egui displays in several sections.
The address bar should offer the equivalent typed command for a reproducible
page. The GUI retains its own tabs, splits, focus, local table state, and
notifications. Logs and background work are scoped to the running application.

## Recent additions

Commands below use the same account and service scope as the matching desktop
operation. Add the relevant authentication and scope flags when selecting a
specific account, tenant, organization, project, or Helix server.

| Desktop operation | CLI equivalent |
| --- | --- |
| Account list, details, registration, rename, or removal | `ct account list`, `show`, `add`, `rename`, or `remove` |
| Account sign-in, replacement, device code, and sign-out | `ct account login` with `--reauthenticate` or `--device-code` when applicable, and `ct account logout` |
| Account credentials and scoped defaults | `ct account credential` and `ct account default` |
| Tracked Azure tenants, aliases, discovery, and directory details | `ct az tenant list`, `add`, `forget`, `alias`, `discover`, and `show` |
| Default Azure tenant | `ct az tenant show-default` to read and `ct az tenant default <tenant>` to set |
| Entra users, My User, and user search | `ct az entra user list`, `show`, `me`, and `search` |
| Entra user profile photo | `ct az entra user photo --file <path> [--user-id <user-id>]`. Omit the ID for the signed-in user. |
| User group memberships and ownerships | `ct az entra group list --for-member <user-id>` and `ct az entra user owned-groups --user-id <user-id>` |
| User application ownership and assignments | `ct az entra user owned-applications --user-id <user-id>` and `application-assignments --user-id <user-id>` |
| Application assignment role and permission details | `ct az entra service-principal app-role-catalogue`, `app-role-assignments`, and `delegated-grants` |
| User Azure role assignments | `ct az entra user azure-role-assignments --user-id <user-id>` |
| Entra groups, properties, search, members, owners, and directory audit | `ct az entra group list`, `show`, `search`, `member list`, `owners`, and `audit-log` |
| Entra group directory and Azure role assignments | `ct az entra group role-assignments --group-id <group-id>` for directory roles and `ct az entra group azure-role-assignments --group-id <group-id>` for Azure RBAC |
| Entra application registration list, search, details, and credential expiry data | `ct az entra application-registration list`, `search`, and `show` |
| Azure DevOps permissions for an Entra user or group | `ct az devops entra-permissions --user <user-id>` or `--group <group-id>`. The scan uses the selected tenant's tracked organizations. Repeat `--org` to narrow it. |
| Tracked Azure DevOps organizations and tenant associations | `ct az devops org list`, `show`, `add`, and `remove` |
| Azure DevOps organization users and license changes | `ct az devops license-entitlement user list`, `show`, and `update` |
| Azure DevOps project list, overview, and work items | `ct az devops project list`, `project show`, and `work-item recent` |
| Azure DevOps project repos, groups, teams, service connections, agent pools, pipelines, and test plans | `ct az devops repo list`, `group list`, `team list`, `service-endpoint list`, `agent pool entitlement list`, `build definition list`, and `test plan list` |
| Tracked Helix servers, defaults, and Innovation Suite URL | `ct helix server list`, `show`, `add`, `remove`, `default`, and `set-innovation-suite-url` |
| Helix work orders and exact-ID lookup | `ct helix work-order list` and `ct helix work-order show <work-order-id>` |
| Helix Shared Ticket Console and ticket details | `ct helix ticket list` and `ct helix ticket show <record-id>` |

Use `--no-cache` on the cached Entra and Azure DevOps inventory reads for fresh
data. Entra group and application registration searches use display-name tokens.
Entra user search supports the desktop's exact lookup through
`--exact-display-name`. Helix ticket details use the same bounded inventory of
open, assigned tickets as the desktop. Increase `--limit` to search more records.

For Azure DevOps organization and project data, copy the desktop command to
reproduce its selected credential. Pass `--tenant <tenant>` to use the tenant's
default Entra account, as the desktop does when opening an associated
organization. The CLI also supports the organization's configured PAT default.
Use `--has-license` to require a fresh entitlement check before a license change.

Photo export writes to a new file at the requested path. Structured output
contains the saved path or `null` when the user has no photo.
