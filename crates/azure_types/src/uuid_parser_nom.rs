use nom::IResult;
use nom::Parser;
use nom::bytes::take;
use nom::combinator::map_res;
use nom::error::ParseError;
use uuid::Uuid;

pub fn parse_uuid_nom<
    'a,
    E: ParseError<&'a str> + nom::error::FromExternalError<&'a str, uuid::Error>,
>(
    i: &'a str,
) -> IResult<&'a str, Uuid, E> {
    let mut parser = map_res(take(36usize), Uuid::parse_str);
    parser.parse(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_uuid() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let (rest, parsed) = parse_uuid_nom::<nom::error::Error<&str>>(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(parsed, Uuid::parse_str(input).unwrap());
    }

    #[test]
    fn test_invalid_uuid_missing_dash() {
        let input = "550e8400e29b-41d4-a716-446655440000";
        let result = parse_uuid_nom::<nom::error::Error<&str>>(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_uuid_non_hex() {
        let input = "550e8400-e29b-41d4-a716-44665544zzzz";
        let result = parse_uuid_nom::<nom::error::Error<&str>>(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_uuid_too_short() {
        let input = "550e8400-e29b-41d4-a716-44665544";
        let result = parse_uuid_nom::<nom::error::Error<&str>>(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_uuid_uppercase() {
        let input = "550E8400-E29B-41D4-A716-446655440000";
        let (rest, parsed) = parse_uuid_nom::<nom::error::Error<&str>>(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(parsed, Uuid::parse_str(input).unwrap());
    }

    #[test]
    fn test_uuid_with_extra_characters() {
        let input = "550e8400-e29b-41d4-a716-446655440000extra";
        let (rest, parsed) = parse_uuid_nom::<nom::error::Error<&str>>(input).unwrap();
        assert_eq!(rest, "extra");
        assert_eq!(
            parsed,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_random_uuid_roundtrip() {
        for _ in 0..10 {
            let random_uuid = Uuid::new_v4();
            let uuid_str = random_uuid.as_hyphenated().to_string();
            let (rest, parsed) = parse_uuid_nom::<nom::error::Error<&str>>(&uuid_str).unwrap();
            assert_eq!(rest, "");
            assert_eq!(parsed, random_uuid);
        }
    }

    #[test]
    fn test_uuid_with_leading_and_trailing_whitespace() {
        let input = " 550e8400-e29b-41d4-a716-446655440000 ";
        let result = parse_uuid_nom::<nom::error::Error<&str>>(input);
        assert!(
            result.is_err(),
            "Parser should not accept leading/trailing whitespace"
        );
    }

    #[test]
    fn test_empty_string() {
        let input = "";
        let result = parse_uuid_nom::<nom::error::Error<&str>>(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_partial_uuid() {
        let input = "550e8400-e29b-41d4";
        let result = parse_uuid_nom::<nom::error::Error<&str>>(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_uuid_with_internal_spaces() {
        let input = "550e8400-e29b-41d4-a716-4466 55440000";
        let result = parse_uuid_nom::<nom::error::Error<&str>>(input);
        assert!(result.is_err());
    }
}
