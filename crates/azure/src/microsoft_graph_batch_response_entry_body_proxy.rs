#![allow(
    clippy::disallowed_types,
    reason = "The batch body proxy and its generated Facet code preserve raw JSON until selecting the success or error shape."
)]

use crate::MicrosoftGraphBatchResponseEntryBody;
use crate::microsoft_graph_batch_error_envelope::MicrosoftGraphBatchErrorEnvelope;
use cloud_terrastodon_command::FromCommandOutput;
use facet_json::RawJson;

#[derive(Clone, Debug, PartialEq, Eq, facet::Facet)]
#[facet(transparent)]
pub struct MicrosoftGraphBatchResponseEntryBodyProxy(pub(crate) RawJson<'static>);

impl<T: FromCommandOutput> TryFrom<MicrosoftGraphBatchResponseEntryBodyProxy>
    for MicrosoftGraphBatchResponseEntryBody<T>
{
    type Error = eyre::Report;

    fn try_from(value: MicrosoftGraphBatchResponseEntryBodyProxy) -> eyre::Result<Self> {
        let raw = value.0;
        if let Ok(envelope) = facet_json::from_str::<MicrosoftGraphBatchErrorEnvelope>(raw.as_str())
        {
            return Ok(Self::Error(envelope.error));
        }
        let value = facet_json::from_str::<T>(raw.as_str()).map_err(|error| {
            eyre::eyre!(
                "failed to deserialize Microsoft Graph batch body as {}: {:?}",
                std::any::type_name::<T>(),
                error
            )
        })?;
        Ok(Self::Success(value))
    }
}

impl<T: FromCommandOutput> TryFrom<&MicrosoftGraphBatchResponseEntryBody<T>>
    for MicrosoftGraphBatchResponseEntryBodyProxy
{
    type Error = eyre::Report;

    fn try_from(value: &MicrosoftGraphBatchResponseEntryBody<T>) -> Result<Self, Self::Error> {
        let json = match value {
            MicrosoftGraphBatchResponseEntryBody::Success(value) => {
                facet_json::to_string(value).map_err(|error| eyre::eyre!("{error:?}"))?
            }
            MicrosoftGraphBatchResponseEntryBody::Error(error) => {
                facet_json::to_string(&MicrosoftGraphBatchErrorEnvelope {
                    error: error.clone(),
                })
                .map_err(|error| eyre::eyre!("{error:?}"))?
            }
        };
        Ok(Self(RawJson::from_owned(json)))
    }
}
