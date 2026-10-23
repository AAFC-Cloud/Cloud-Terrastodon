use crate::AzurePrincipalArgument;
use crate::EntraPrincipal;
use crate::EntraPrincipalId;
use crate::EntraUserId;
use crate::UserPrincipalName;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Report;
use eyre::Result;
use eyre::bail;
use std::borrow::Cow;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

/// An object ID or validated UPN selecting an Entra user.
///
/// Microsoft Graph's [get user](https://learn.microsoft.com/en-us/graph/api/user-get?view=graph-rest-1.0)
/// operation accepts either identifier. Text input accepts a UUID or UPN, with
/// surrounding whitespace removed before parsing. Display names require search.
/// Parsing and scalar deserialization own their validated value; conversion from
/// a typed reference borrows it until [`Self::into_owned`] is requested.
#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
#[facet(proxy = String)]
#[repr(C)]
pub enum EntraUserArgument<'a> {
    ObjectId(Cow<'a, EntraUserId>),
    UserPrincipalName(Cow<'a, UserPrincipalName>),
}

impl EntraUserArgument<'_> {
    /// Retain this selection independently of any borrowed caller-owned value.
    pub fn into_owned(self) -> EntraUserArgument<'static> {
        match self {
            Self::ObjectId(id) => EntraUserArgument::ObjectId(Cow::Owned(id.into_owned())),
            Self::UserPrincipalName(upn) => {
                EntraUserArgument::UserPrincipalName(Cow::Owned(upn.into_owned()))
            }
        }
    }
}

impl From<EntraUserId> for EntraUserArgument<'_> {
    fn from(value: EntraUserId) -> Self {
        Self::ObjectId(Cow::Owned(value))
    }
}

impl<'a> From<&'a EntraUserId> for EntraUserArgument<'a> {
    fn from(value: &'a EntraUserId) -> Self {
        Self::ObjectId(Cow::Borrowed(value))
    }
}

impl From<UserPrincipalName> for EntraUserArgument<'_> {
    fn from(value: UserPrincipalName) -> Self {
        Self::UserPrincipalName(Cow::Owned(value))
    }
}

impl<'a> From<&'a UserPrincipalName> for EntraUserArgument<'a> {
    fn from(value: &'a UserPrincipalName) -> Self {
        Self::UserPrincipalName(Cow::Borrowed(value))
    }
}

impl FromStr for EntraUserArgument<'_> {
    type Err = Report;

    fn from_str(value: &str) -> Result<Self> {
        let value = value.trim();
        if let Ok(id) = value.parse::<EntraUserId>() {
            Ok(Self::from(id))
        } else {
            value.parse::<UserPrincipalName>().map(Self::from)
        }
    }
}

impl TryFrom<String> for EntraUserArgument<'_> {
    type Error = Report;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl TryFrom<&str> for EntraUserArgument<'_> {
    type Error = Report;

    fn try_from(value: &str) -> Result<Self> {
        value.parse()
    }
}

impl<'a> TryFrom<AzurePrincipalArgument<'a>> for EntraUserArgument<'a> {
    type Error = Report;

    fn try_from(value: AzurePrincipalArgument<'a>) -> Result<Self> {
        match value {
            AzurePrincipalArgument::Id(id) => match id {
                Cow::Borrowed(EntraPrincipalId::UserId(id)) => Ok(Self::from(id)),
                Cow::Owned(EntraPrincipalId::UserId(id)) => Ok(Self::from(id)),
                Cow::Borrowed(EntraPrincipalId::Unknown(id)) => {
                    Ok(Self::from(EntraUserId::new(*id)))
                }
                Cow::Owned(EntraPrincipalId::Unknown(id)) => Ok(Self::from(EntraUserId::new(id))),
                _ => {
                    bail!("Known group and service-principal IDs cannot identify a user");
                }
            },
            AzurePrincipalArgument::Name(name) => name.parse(),
            AzurePrincipalArgument::Principal(principal) => match principal {
                Cow::Borrowed(EntraPrincipal::User(user)) => Ok(Self::from(&user.id)),
                Cow::Owned(EntraPrincipal::User(user)) => Ok(Self::from(user.id)),
                _ => {
                    bail!("A user argument requires a user principal");
                }
            },
        }
    }
}

impl Display for EntraUserArgument<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ObjectId(id) => id.fmt(f),
            Self::UserPrincipalName(name) => name.fmt(f),
        }
    }
}

impl From<&EntraUserArgument<'_>> for String {
    fn from(value: &EntraUserArgument<'_>) -> Self {
        value.to_string()
    }
}

impl<'input> Arbitrary<'input> for EntraUserArgument<'static> {
    fn arbitrary(input: &mut Unstructured<'input>) -> arbitrary::Result<Self> {
        if bool::arbitrary(input)? {
            Ok(Self::from(EntraUserId::arbitrary(input)?))
        } else {
            Ok(Self::from(UserPrincipalName::arbitrary(input)?))
        }
    }
}

cloud_terrastodon_registry::register_thing!(EntraUserArgument<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraUserArgument<'static>);

#[cfg(test)]
mod tests {
    use super::EntraUserArgument;
    use crate::AzurePrincipalArgument;
    use crate::EntraGroupId;
    use crate::EntraPrincipalId;
    use crate::EntraUserId;
    use crate::UserPrincipalName;
    use std::borrow::Cow;
    use std::ptr;

    #[test]
    fn scalar_selection_normalizes_input_and_validates_upns() {
        let id: EntraUserId = "11111111-1111-4111-8111-111111111111".parse().unwrap();
        let argument: EntraUserArgument<'static> = format!(" {id} ").parse().unwrap();
        assert_eq!(argument, EntraUserArgument::from(id));
        let argument: EntraUserArgument<'static> = " Alex_example.invalid#EXT#@tenant.invalid "
            .parse()
            .unwrap();
        assert_eq!(
            argument.to_string(),
            "Alex_example.invalid#EXT#@tenant.invalid"
        );
        for invalid in [
            "",
            "Alex Smith",
            "alex",
            "alex@",
            "a@b@c",
            "a b@example.invalid",
        ] {
            assert!(
                invalid.parse::<EntraUserArgument<'_>>().is_err(),
                "{invalid:?}"
            );
        }
    }

    #[test]
    fn a_known_group_id_cannot_be_reinterpreted_as_a_user_id() {
        let id: EntraGroupId = "11111111-1111-4111-8111-111111111111".parse().unwrap();
        let known_group = AzurePrincipalArgument::from(EntraPrincipalId::from(id));
        assert!(EntraUserArgument::try_from(known_group).is_err());
        let unclassified = AzurePrincipalArgument::from(EntraPrincipalId::new(*id.as_ref()));
        assert!(EntraUserArgument::try_from(unclassified).is_ok());
    }

    #[test]
    fn typed_references_borrow_until_the_selection_takes_ownership() {
        let owned_upn: EntraUserArgument<'static> = {
            let upn: UserPrincipalName = "alex@example.invalid".parse().unwrap();
            let argument = EntraUserArgument::from(&upn);
            let EntraUserArgument::UserPrincipalName(Cow::Borrowed(borrowed)) = &argument else {
                panic!("a typed UPN reference should remain borrowed");
            };
            assert!(ptr::eq(*borrowed, &upn));
            argument.into_owned()
        };
        assert_eq!(owned_upn.to_string(), "alex@example.invalid");

        let owned_id: EntraUserArgument<'static> = {
            let id =
                EntraPrincipalId::UserId("11111111-1111-4111-8111-111111111111".parse().unwrap());
            let argument = EntraUserArgument::try_from(AzurePrincipalArgument::from(&id))
                .expect("a known user ID remains a valid user selection");
            let EntraUserArgument::ObjectId(Cow::Borrowed(borrowed)) = &argument else {
                panic!("a user ID inside a borrowed principal ID should remain borrowed");
            };
            let EntraPrincipalId::UserId(original) = &id else {
                unreachable!();
            };
            assert!(ptr::eq(*borrowed, original));
            argument.into_owned()
        };
        assert_eq!(owned_id.to_string(), "11111111-1111-4111-8111-111111111111");
    }
}
