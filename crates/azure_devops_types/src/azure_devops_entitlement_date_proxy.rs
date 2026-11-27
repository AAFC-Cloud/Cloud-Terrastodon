use facet::Facet;
use jiff::Timestamp;
use jiff::tz::Offset;

/// An optional entitlement date on the wire, where the minimum date represents an unset value.
#[derive(Facet)]
#[facet(transparent)]
pub(crate) struct AzureDevOpsEntitlementDateProxy(Option<String>);

impl TryFrom<AzureDevOpsEntitlementDateProxy> for Option<Timestamp> {
    type Error = jiff::Error;

    fn try_from(value: AzureDevOpsEntitlementDateProxy) -> Result<Self, Self::Error> {
        match value.0.as_deref() {
            Some(date) => parse_entitlement_timestamp(date),
            None => Ok(None),
        }
    }
}

impl From<&Option<Timestamp>> for AzureDevOpsEntitlementDateProxy {
    fn from(value: &Option<Timestamp>) -> Self {
        Self(value.as_ref().map(ToString::to_string))
    }
}

pub(crate) fn parse_entitlement_timestamp(value: &str) -> Result<Option<Timestamp>, jiff::Error> {
    // PATCH results can use the uninitialized .NET date without an offset.
    if value == "0001-01-01T00:00:00" {
        return Ok(None);
    }

    let timestamp = value.parse::<Timestamp>()?;
    let utc = Offset::UTC.to_datetime(timestamp);
    // The Get and Add examples report year-1 sentinels at different hours.
    Ok((utc.year() != 1 || utc.day_of_year() != 1).then_some(timestamp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_date_preserves_unknowns_and_requires_offsets_for_real_dates() -> eyre::Result<()> {
        for wire in [
            None,
            Some("0001-01-01T00:00:00"),
            Some("0001-01-01T00:00:00Z"),
        ] {
            let date = Option::<Timestamp>::try_from(AzureDevOpsEntitlementDateProxy(
                wire.map(str::to_owned),
            ))?;
            assert_eq!(date, None);
        }
        let created = "2023-10-05T12:34:56+02:00";
        let date = Option::<Timestamp>::try_from(AzureDevOpsEntitlementDateProxy(Some(
            created.to_owned(),
        )))?;
        assert_eq!(date, Some("2023-10-05T10:34:56Z".parse()?));
        assert!(parse_entitlement_timestamp("2023-10-05T12:34:56").is_err());
        Ok(())
    }
}
