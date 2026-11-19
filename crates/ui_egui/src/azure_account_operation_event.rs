use cloud_terrastodon_auth::AccountSignInPrompt;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use teamy_cancellation::CancellationToken;
use teamy_tracing::DiagnosticReport;

pub(crate) enum AzureAccountOperationEvent {
    Started(CancellationToken),
    Prompt(AccountSignInPrompt),
    RefreshingCatalogue,
    Finished {
        outcome: Result<Option<CloudTerrastodonAccountId>, DiagnosticReport>,
        provider_closed: bool,
        catalogue: Result<CloudTerrastodonAccountCatalogue, DiagnosticReport>,
    },
}
