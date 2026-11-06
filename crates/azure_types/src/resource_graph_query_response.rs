use crate::RawResourceGraphQueryResponse;
use eyre::Context;
use eyre::Result;
use facet::Facet;

/// One Resource Graph response with table rows converted to typed records.
///
/// This is the library's decoded form of the
/// [Resource Graph query response](https://learn.microsoft.com/en-us/rest/api/azureresourcegraph/resourcegraph/resources/resources?view=rest-azureresourcegraph-resourcegraph-2022-10-01#queryresponse).
/// It contains data only; decoding bounds belong to the conversion.
#[derive(Debug)]
pub struct ResourceGraphQueryResponse<T> {
    pub count: u64,
    pub data: Vec<T>,
    pub skip_token: Option<String>,
    pub total_records: u64,
    pub truncated: bool,
}

impl<T> TryFrom<RawResourceGraphQueryResponse> for ResourceGraphQueryResponse<T>
where
    T: Facet<'static>,
{
    type Error = eyre::Error;

    fn try_from(value: RawResourceGraphQueryResponse) -> Result<Self> {
        Ok(Self {
            count: value.count,
            data: value.data.into_records().wrap_err("transforming data")?,
            skip_token: value.skip_token,
            total_records: value.total_records,
            truncated: value
                .truncated
                .parse()
                .wrap_err("parsing boolean named 'truncated'")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArbitraryJson;
    use crate::ResourceGraphColumn;
    use crate::ResourceGraphData;

    #[test]
    fn transforms_resource_group_rows_with_string_backed_fields() -> eyre::Result<()> {
        fn column(name: &str) -> ResourceGraphColumn {
            ResourceGraphColumn {
                name: name.to_string(),
                kind: "dynamic".to_string(),
            }
        }

        let raw = RawResourceGraphQueryResponse {
            count: 1,
            data: ResourceGraphData {
                columns: vec![
                    column("id"),
                    column("tenant_id"),
                    column("location"),
                    column("managed_by"),
                    column("name"),
                    column("properties"),
                    column("tags"),
                    column("subscription_name"),
                ],
                rows: vec![vec![
                    ArbitraryJson::from_text(
                        "/subscriptions/00000000-0000-0000-0000-000000000000/resourceGroups/example-rg",
                    ),
                    ArbitraryJson::from_text("00000000-0000-0000-0000-000000000000"),
                    ArbitraryJson::from_text("eastus"),
                    ArbitraryJson::from_text(""),
                    ArbitraryJson::from_text("example-rg"),
                    facet_json::from_str(r#"{"provisioningState":"Succeeded"}"#)?,
                    ArbitraryJson::object(),
                    ArbitraryJson::from_text("Example Subscription"),
                ]],
            },
            skip_token: None,
            truncated: "false".to_string(),
            total_records: 1,
        };

        let response: ResourceGraphQueryResponse<crate::ResourceGroup> = raw.try_into()?;

        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].name.to_string(), "example-rg");
        assert_eq!(
            response.data[0].properties["provisioningState"],
            "Succeeded"
        );
        Ok(())
    }
}
