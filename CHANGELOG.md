# v0.29.0

- Rename `fetch_groups` to `fetch_all_groups`
- Add `--json` flag to write logs to a `.jsonl` file
- Add debug warning when picker tui elements contain `\t`
- Fix choice alignment in "copy azurerm backend" action
- Introduce compute sku and vm image structs and helper fetcher functions
- Fix picker TUI multiline support
- Fix picker TUI ctrl+c support
- Make app exit happily on ctrl+c or esc on main menu
- Add `azure audit` and `azure-dev-ops audit` commands

-- AI overview

Of course, here is a detailed English changelog for v0.29.0 based on the provided changes.

---

# v0.29.0

### Major Features

- **Added `azure audit` and `azure-devops audit` Commands**
  - The `azure audit` command scans your Azure environment for potential issues, including:
    - Role assignments that point to deleted principals (users, groups, service principals).
    - Service principals with expired or soon-to-expire credentials.
  - The `azure-devops audit` command checks your Azure DevOps organization for possible optimizations, such as:
    - Users with paid licenses who have never logged in.
    - License entitlements for users who no longer exist in Entra ID.
- **Added `--json` Flag for Logging**
  - You can now use the `--json` flag to write structured logs to a timestamped `.jsonl` file, making parsing and debugging easier.
- **Enhanced Resource Owner Discovery**
  - The interactive "Find Resource Owners" command can now trace ownership through Azure DevOps Service Endpoints and their associated projects.
- **New Types and Helpers for Azure Compute SKUs**
  - Introduced new structs for compute SKUs, virtual machine sizes, and pricing.
  - Added helper functions to fetch all SKU and VM pricing information, facilitating cost and capacity analysis.

### Improvements and Bug Fixes

- **Terminal Picker UI Enhancements**
  - Fixed text alignment in the "copy azurerm backend" action for better readability.
  - Improved support for multiline display in the picker.
  - The application now exits gracefully when the user presses `Ctrl+C` or `Esc` in the main menu or pickers.
  - Added a debug warning when picker items contain tab characters (`\t`), which can cause rendering issues.
- **Code Refactoring**
  - Renamed `fetch_groups` to `fetch_all_groups` for clarity.
  - The code for fetching group members and owners has been split into its own modules for better organization.
  - Implemented batch fetching for group members via the Microsoft Graph API, improving performance.
- **Improved Type Safety**
  - The `passwordCredentials` and `keyCredentials` fields on the `ServicePrincipal` model are now strongly typed for better security and reliability.
- **Documentation**
  - Comprehensively updated the `README.md` and `README.fr_ca.md` files with new sections on Installation, Usage, Caching, and a link to a video demonstration.

### Internal

- Updated several dependencies, including `sysinfo`, `eframe`, `egui`, and the `windows` crate.

---

Voici une version étoffée du journal des modifications pour la v0.29.0, basée sur les changements que vous avez fournis.

---

# v0.29.0

### Caractéristiques Majeures

- **Ajout des commandes `azure audit` et `azure-devops audit`**
  - La commande `azure audit` analyse votre environnement Azure pour y déceler des problèmes potentiels, notamment :
    - Les attributions de rôles qui pointent vers des principaux (utilisateurs, groupes, principaux de service) supprimés.
    - Les principaux de service avec des informations d'identification expirées ou sur le point d'expirer.
  - La commande `azure-devops audit` vérifie votre organisation Azure DevOps pour y déceler des optimisations possibles, telles que :
    - Les utilisateurs disposant de licences payantes qui ne se sont jamais connectés.
    - Les droits de licence pour les utilisateurs qui n'existent plus dans Entra ID.
- **Ajout du drapeau `--json` pour la journalisation**
  - Vous pouvez maintenant utiliser le drapeau `--json` pour écrire des journaux structurés dans un fichier `.jsonl` avec horodatage, ce qui facilite l'analyse et le débogage.
- **Amélioration de la découverte des propriétaires de ressources**
  - La commande interactive "Trouver les propriétaires de ressources" peut désormais suivre la propriété via les points de terminaison de service Azure DevOps et leurs projets associés.
- **Nouveaux types et assistants pour les SKU de calcul Azure**
  - Introduction de nouvelles structures pour les SKU de calcul, les tailles de machines virtuelles et les prix.
  - Ajout de fonctions d'assistance pour récupérer toutes les informations relatives aux SKU et aux prix des machines virtuelles, facilitant ainsi les analyses de coûts et de capacités.

### Améliorations et Corrections de Bugs

- **Améliorations de l'interface utilisateur du sélecteur de terminal**
  - Correction de l'alignement du texte dans l'action "copier le backend azurerm" pour une meilleure lisibilité.
  - Amélioration de la prise en charge de l'affichage multiligne dans le sélecteur.
  - L'application se ferme maintenant correctement lorsque l'utilisateur appuie sur `Ctrl+C` ou `Échap` dans le menu principal ou les sélecteurs.
  - Ajout d'un avertissement de débogage lorsque les éléments du sélecteur contiennent des caractères de tabulation (`\t`), qui peuvent causer des problèmes de rendu.
- **Refactorisation du code**
  - Renommage de `fetch_groups` en `fetch_all_groups` pour plus de clarté.
  - Le code pour récupérer les membres et les propriétaires de groupes a été séparé dans ses propres modules pour une meilleure organisation.
  - Implémentation de la récupération par lots pour les membres de groupe via l'API Microsoft Graph, améliorant ainsi les performances.
- **Amélioration de la sécurité des types**
  - Les champs `passwordCredentials` et `keyCredentials` sur le modèle `ServicePrincipal` sont maintenant fortement typés pour une meilleure sécurité et fiabilité.
- **Documentation**
  - Mise à jour complète des fichiers `README.md` et `README.fr_ca.md` avec de nouvelles sections sur l'installation, l'utilisation, la mise en cache, et un lien vers une vidéo de démonstration.

### Interne

- Mise à jour de plusieurs dépendances, y compris `sysinfo`, `eframe`, `egui`, et la caisse `windows`.

---

---

# v0.28.0

- Switch from using `fzf` to `PickerTui` everywhere
- Update `PickerTui` return types for `pick_one` and `pick_many` to return `PickResult<T>`
- Add `PickerTui::from` for better type inference
- Remove `fzf` module containing `pick` and `pick_many` in favour of `PickerTui`

# v0.27.0

- Added `KeyVaultSecretId` type
- Added `KeyVaultSecretVersionId` type
- Update `KeyVaultSecret` to use new `KeyVaultSecretId` type
- Impl `Ord` for `KeyVaultId`
- Added browse storage accounts action
- Add `GovernanceRoleAssignmentMemberType::Direct` variant
- Add `PrincipalCollection` type and change `fetch_all_principals` to return it
- Add `UnifiedRoleDefinition` and `UnifiedRoleAssignment` types for Entra RBAC
- Move `AccessToken` type from `cloud_terrastodon_credentials` to `cloud_terrastodon_azure_types`
- Add jwt decoding to `cloud_terrastodon_credentials` (TODO: fix fn return instead of just printing)
- Published `cloud_terrastodon` crate to re-export other `cloud_terrastodon_*` crates

# v0.26.0

- Add `fetch_all_key_vaults` fn
- Add `KeyVault` and `KeyVaultId` and `KeyVaultProperties` and `KeyVaultName` types
- Remove storage account duplicate properties already exposed by the id
- Rename PIM role assignment stuff to `GovernanceRoleAssignment`

# v0.25.0

- Revert virtual network address space back to `Ipv4Network`
- Revamp fetch_all_policy_assignments to use resource graph

# v0.24.0

- Fix command cache busting
- Add `AzureDevOpsDefaultOrganizationUrlTui` to `cloud_terrastodon_azure_devops`
- Add `MessageBoxTui` to `cloud_terrastodon_user_input`
- Add `cloud_terrastodon_credentials` crate for exploration into using our own REST client instead of `az rest` and `az devops invoke` due to auth being annoying
- Fix route table deserialization using new `AddressPrefix` type

# v0.23.0

- Tracing now outputs to stderr
- Add `impl FromStr for StorageAccountId`
- Add `AzureDevOpsOrganizationUrl` parameter to azure devops functions

# v0.22.0

- Add `PickerTui` to `cloud_terrastodon_user_input`

# v0.21.0

- Fix conditional access policy struct where included/excluded applications aren't always UUIDs

# v0.20.0

- Add `fetch_azure_devops_user_license_entitlements()` function to retrieve Azure DevOps user entitlements
- Add "Invalid combination of arguments" to list of fixable errors for GenerateConfigOutHelper
- Add `get_azure_devops_user_onboarding_statuses(user_emails)` function
- Reduce log level for fetch helpers to DEBUG from INFO
- Updated dependencies via `cargo update`

# v0.19.0

- `cloud_terrastodon clean` no longer shows warnings for directories not present
- Add `--debug` argument always for azure CLI commands

# v0.18.0

- Add stronger types for azure devops service endpoint
- Add cache bust when failed to find default azure devops project or organization
- Add automatic no_space conversion for command cache keys

# v0.17.0

- Add conditional access policy stuff
- Introduce variants for RoleDefinitionId
- Add virtual network peering and name types

# v0.16.1

- Fix subnet properties route table reference using `RouteTable`, now is `RouteTableId`

# v0.16.0

- Fix subnet types to not use optionals
- Fix subnet `addressPrefixes` and `addressPrefix` variant deserializing
- Fix subnet id constructors and serialization
- Remove interior mutability from Name types to prevent subverting validation after construction
- Make tags deserialize into `HashMap` instead of `Optional<HashMap>`
- Fix subnet properties route table reference to use RouteTableId instead of String

# v0.15.0

- Add `cloud_terrastodon terraform audit --recursive` command
- Add virtual network and subnetwork and route table types

# v0.14.0

- Add azure devops groups and teams and membership helpers for each
- Add `browse azure devops projects` command
- Add `browse azure devops teams` command

# v0.13.0

- Change `Subscription { name: String }` to `Subscription { name: SubscriptionName }`
- Change `ResourceGroup { name: String }` to `ResourceGroup { name: ResourceGroupName }`
- Add `ScopeImplKind::StorageAccount` support in `name_lookup_helper::fetch_names_for`

# v0.12.0

- Fix trait bounds on `ResourceGroupId::try_new` to allow passing `&str`
- Fix trait bounds on `StorageAccountId::try_new` to allow passing `&str`
- Fix trait bounds on `SubscriptionId::try_new` to allow passing `&str`
- Added `impl TryFrom<&str> for ResourceGroupName`
- Added `impl TryFrom<&str> for StorageAccountName`
- Added `impl TryFrom<&str> for SubscriptionName`
- Added `impl TryFrom<&str> for StorageAccountBlobContainerName`

# v0.11.0

- Impl `Arbitrary` for ResourceGroupId
- Impl `Arbitrary` for StorageAccountId
- Add `StorageAccountId::new` and `StorageAccountId::try_new`
- Add `ResourceGroupId::try_new`
- Add `SubscriptionId::try_new`

# v0.10.0

- Rename `HasScope::scope(&self)` to `AsScope::as_scope(&self)`
- Rename `Scope::as_scope(&self)` to `Scope::as_scope_impl(&self)`
- Change `Resource::id` from `CompactString` to `ScopeImpl`
- Change `RoleAssignment::scope` from `CompactString` to `ScopeImpl`
- Change `ScopeImpl::try_from_expanded` to return `Result<Self, Infallible>`
- Add `impl<T> From<T> for ScopeImpl where T: AsRef<str>`
- Add `fetch_storage_account_blob_container_names(id: &StorageAccountId) -> HashSet<StorageAccountBlobContainerName>`

# v0.9.0

- The Great Big ID Rework - instead of storing simple strings to the resource, I fully parse the ID into its components.
- Fix policy import builder
- Fix name sanitization when reflowing Terraform workspaces
- Truncate command output when displaying errors, only shows first and last 500 lines

# v0.8.0

- Remove `_core` suffix from crates
- Separate resoure types to separate crate to maximize cache hits
- Flatten repository structure
- Add `ct terraform import` command
- Published `cloud_terrastodon_*` crates to crates.io

# v0.7.1

- Fix invalid assumption from role eligibility schedule ID parsing, should fix ct pim activate for azurerm

# v0.7.0

- Progress on cloud_terrastodon dump-everything to export devops projects and resource groups
- static analysis of terraform required provider version

# v0.6.0

- Add oauth2 scope management
- Fix errors when used with account in tenant with no management groups
- Add query option to fzfargs
- Use mutex to prevent multiple sign-ins when auth failed in concurrent requests
- Add bulk user id lookup

# v0.5.0

- Add security group and role assignment imports to `write-all-imports`
- Add interactive option for running `write-all-imports`
- Fix deduplication logic when writing tf files
- Fix unknown scopes getting interpreted as my test type
- Fix group imports dynamic_membership conflicting with generated member list

# v0.4.0

- Fix PIM role activation happening twice when two role assignments present for the same role
- Add wizard for generating import blocks
- Add `tf plan` action
- Remove default attributes when processing generated HCL

# v0.3.0

- Fix policy remediation not providing scope leading to 0 resources being remediated
- Add `cloud_terrastodon copy-results ./whatever` command

# v0.2.0

- Fix terminal colours in default terminal opened when double clicking the exe
    - https://stackoverflow.com/questions/78741673/colors-not-working-on-default-terminal-for-release-rust-exe/78741674#78741674
- Add app icon
- Clean up non-interactive usage scenarios (see: `cloud_terrastodon --help`)
- Linux (Ubuntu) working
- First GitHub release

# v0.1.1

- Fix "Justification:" prompt not showing when activating PIM roles