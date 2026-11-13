# Storage account anonymous-access audit

Run the storage audit independently or as part of the Azure audit:

```text
ct az storage account audit --tenant <tenant-or-alias> --account <account-alias> --clean
ct az audit --tenant <tenant-or-alias> --account <account-alias> --clean
```

The commands use the normal account authentication selection, including an
optional `--auth-source` override and deliberately configured defaults. The
standard output format selection supports text, JSON, and Facet Pretty.

Inspect storage accounts independently of the audit with:

```text
ct az storage account list --tenant <tenant-or-alias> --account <account-alias>
ct az storage account show --tenant <tenant-or-alias> --account <account-alias> --storage-account <storage-account-arm-id>
```

`list` shares the audit's Resource Graph account inventory. `show` reads the
selected account directly through ARM, including its managed identities and
provenance when returned. Both support `--no-cache`: listing refreshes the
selected tenant inventory, while showing refreshes only that account's details.

Inspect the underlying container properties directly with:

```text
ct az storage blob container list --tenant <tenant-or-alias> --account <account-alias> --storage-account <storage-account-arm-id>
ct az storage blob container show --tenant <tenant-or-alias> --account <account-alias> --storage-account <storage-account-arm-id> --container <container-name>
```

Both commands support `--no-cache`. Listing refreshes all container inventory
pages for the selected account; showing refreshes only the selected container's
properties. These commands return management-plane configuration, including
`publicAccess`, without reading blob contents.

The audit identifies storage accounts whose `allowBlobPublicAccess` is explicitly
`true` and whose complete live container inventory contains only `publicAccess:
None`. An account with no containers also qualifies. Containers with `Blob` or
`Container` access exclude the account from this finding. FileStorage accounts
have no blob service and are excluded.

An enabled account setting alone does not make blob data public. Microsoft
documents the [account and container access settings separately](https://learn.microsoft.com/en-us/azure/storage/blobs/anonymous-read-access-prevent#about-anonymous-read-access).
This finding identifies an unused configuration capability; it does not test
network reachability, static website endpoints, SAS tokens, or actual anonymous
traffic, and it makes no changes to Azure.

Each finding includes the account identity, anonymous-access setting, public
network access, network rules, tags, and observed container configurations.
Network restrictions provide context; they do not suppress this configuration
finding. Inconclusive findings preserve missing account settings, failed
container inventories, and unrecognized container access levels. A read failure
is never interpreted as an empty or private account. Cancellation aborts the
audit. Accounts are limited to the inventory visible to the selected identity;
an empty result does not establish tenant-wide permissions or coverage.

The audit reuses `AzureStorageAccountListRequest` and its cached inventory.
`AzureStorageAccount.properties` contains `AzureStorageAccountProperties`, with
typed access controls, encryption, endpoints, replication, and other documented
settings. There is no separate access-settings projection or cache. Containers use
the [ARM Blob Containers List API, version 2025-06-01](https://learn.microsoft.com/en-us/rest/api/storagerp/blob-containers/list?view=rest-storagerp-2025-06-01),
following continuation pages without requesting soft-deleted containers or blob
contents. Unknown response vocabulary is exposed by debug decoding and preserved
by release decoding; it is never assumed private by the audit.

Without `--clean`, normal cache freshness and reuse apply. `--clean` refreshes
the account inventory first, then invalidates each account's container pages
before evaluation. Aggregate audit invalidation completes before concurrent audit
activities start. Refresh failures stop the operation; they never silently fall
back to old findings. Results are observations, not an atomic snapshot across
Resource Graph and ARM.

See [issue #48](https://github.com/AAFC-Cloud/Cloud-Terrastodon/issues/48).
