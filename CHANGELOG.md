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