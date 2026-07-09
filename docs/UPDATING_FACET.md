Cloud Terrastodon uses [Cargo.toml](../Cargo.toml) to declare dependencies.
Cloud Terrastodon depends on `TeamDman/facet`, a fork of `facet-rs/facet`, and is pinned to a git revision rather than a published release.
Cloud Terrastodon has a transitive `facet` dependency through `teamy-mft` which is also pinned to a git revision.

When we perform modifications to `TeamDman/facet`, we must ensure that we update the Cargo.toml git revision pin in addition to updating the `teamy-mft` git repo to have a revision using the same version of `TeamDman/facet` to ensure that there is not two copies of `facet` in the Cloud Terrastodon dependency graph.

For making changes to `facet` and `teamy-mft`, try investigating these paths:

- ~\source\repos\facet
- ~\source\repos\teamy-mft

When working on `facet`, the local `main` branch is tracking `mine/main`. We want new work to be performed against the latest `origin/main` so we should fetch and branch off of that for new work. The `mine/main` branch contains commits merging PRs I have submitted to the upstream `facet-rs` repository.

So, when modifying `facet`, we:
- Fetch origin
- Create a feature branch from `origin/main`
- Perform the changes
- Merge `origin/main` into `main` to bring it up to date to make the next merge easier
- Merge feature branch into `main`
- Push
- Cloud Terrastodon update pinned `facet` git revision
- `teamy-mft` repo update pinned `facet` git revision
- Cloud Terrastodon update pinned `teamy-mft` git revision

Additionally, the `facet` repo has broken git hooks.
We must use `--no-verify` when committing and pushing.

Once we have a feature branch we have verified as working with Cloud Terrastodon, we have the option to contribute upstream:
1. Create an issue that clearly describes the failure mode, including sample code and sample output
2. Create a pull request that cites the issue, including before and after outputs.

When submitting commentary like issues, pull requests, and comments to GitHub, use the `gh` cli.
A temporary markdown file must be created for the user to review the content of the submission before you are allowed to publish.

Submissions must include a disclaimer that content was written with LLM assistance, including the name and precise version of the LLM.