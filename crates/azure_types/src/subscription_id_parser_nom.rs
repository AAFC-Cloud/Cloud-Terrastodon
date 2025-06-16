use crate::prelude::SubscriptionId;
use crate::prelude::parse_uuid_nom;
use nom::IResult;
use nom::Parser;
use nom::bytes::complete::tag;
use nom::bytes::complete::tag_no_case;
use nom::combinator::map;
use nom_language::error::VerboseError;

pub fn parse_subscription_id_nom(i: &str) -> IResult<&str, SubscriptionId, VerboseError<&str>> {
    let (i, _) = tag("/")(i)?;
    let (i, _) = tag_no_case("subscriptions")(i)?;
    let (i, _) = tag("/")(i)?;
    let mut parser = map(parse_uuid_nom, SubscriptionId::new);
    parser.parse(i)
}

#[cfg(test)]
mod test {
    use crate::prelude::parse_subscription_id_nom;
    use nom::Parser;
    use nom::combinator::all_consuming;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        let subscription_id = "/subscriptions/11112222-3333-4444-aaaa-bbbbccccdddd";
        let mut parser = all_consuming(parse_subscription_id_nom);
        let (rest, subscription_id) = parser.parse(subscription_id)?;
        assert_eq!(rest, "");
        dbg!(subscription_id);
        Ok(())
    }

    #[test]
    pub fn preview_error() -> eyre::Result<()> {
        let subscription_id = "bruh";
        let mut parser = all_consuming(parse_subscription_id_nom);
        let result = parser.parse(subscription_id);
        assert!(result.is_err());
        let err = result.unwrap_err();
        println!("Error: {err:?}");
        Ok(())
    }
}
