When publishing multiple crates in quick succession, you may see the following error

```
cloud-terrastodon\crates\azure on  HEAD (5a00c4d) is 📦 v0.2.0 via 🦀 v1.88.0-nightly 
❯ cargo publish                               
    Updating crates.io index
   Packaging cloud_terrastodon_azure v0.2.0 (C:\Users\phillipsdo\source\repos\cloud-terrastodon\crates\azure)
    Updating crates.io index
error: failed to prepare local package for uploading

Caused by:
  failed to select a version for the requirement `cloud_terrastodon_azure_types = "^0.2.0"`
  candidate versions found which didn't match: 0.1.0
  location searched: crates.io index
  required by package `cloud_terrastodon_azure v0.2.0 (C:\Users\phillipsdo\source\repos\cloud-terrastodon\crates\azure)`
```

(I published v0.2.0 a few minutes ago, and it is visible on the site but fails when trying to publish something that depends on it)

You can force an index update by running `cargo update --dry-run`