#![allow(dead_code, unused, unused_imports)]
use uuid::Uuid;
use winnow::ascii::alpha1;
use winnow::combinator::preceded;
use winnow::combinator::terminated;
use winnow::prelude::*;
use winnow::token::literal;
use winnow::token::take;

use crate::prelude::SubscriptionId;

// Helper function to parse a UUID using winnow
fn parse_uuid_winnow<'a>(input: &mut &'a str) -> winnow::Result<Uuid> {
    take(36usize)
        .verify_map(|s: &str| Uuid::parse_str(s).ok())
        .parse_next(input)
}

// // Helper function for case-insensitive literal matching
// fn literal_no_case<'a>(expected: &'static str) -> impl Parser<&'a str, &'a str, winnow::error::ContextError> {
//     todo!("uncomment lol");

//     // move |input: &mut &'a str| {

//     //     let start = *input;
//     //     if input.len() < expected.len() {
//     //         return Err(winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()));
//     //     }
//     //     let (prefix, rest) = input.split_at(expected.len());
//     //     if prefix.eq_ignore_ascii_case(expected) {
//     //         *input = rest;
//     //         Ok(prefix)
//     //     } else {
//     //         Err(winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()))
//     //     }
//     // }
// }

pub fn parse_subscription_id_winnow<'a>(input: &mut &'a str) -> winnow::Result<SubscriptionId> {
    todo!()
    // preceded(
    //     (literal("/"), literal_no_case("subscriptions"), literal("/")),
    //     parse_uuid_winnow.map(SubscriptionId::new)
    // ).parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::eyre;
    use winnow::Parser;

    #[test]
    fn test_valid_subscription_id() -> eyre::Result<()> {
        let input = "/subscriptions/11112222-3333-4444-aaaa-bbbbccccdddd";
        let result = parse_subscription_id_winnow
            .parse(input)
            .map_err(|e| eyre!("Failed to parse subscription id: {e:#}"))?;
        println!("Parsed subscription ID: {:?}", result);
        Ok(())
    }

    #[test]
    fn test_case_insensitive_subscriptions() -> eyre::Result<()> {
        let test_cases = vec![
            "/subscriptions/11112222-3333-4444-aaaa-bbbbccccdddd",
            "/SUBSCRIPTIONS/11112222-3333-4444-aaaa-bbbbccccdddd",
            "/Subscriptions/11112222-3333-4444-aaaa-bbbbccccdddd",
            "/sUbScRiPtIoNs/11112222-3333-4444-aaaa-bbbbccccdddd",
        ];

        for input in test_cases {
            let result = parse_subscription_id_winnow.parse(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            println!("Successfully parsed: {}", input);
        }
        Ok(())
    }

    #[test]
    fn test_uppercase_uuid() -> eyre::Result<()> {
        let input = "/subscriptions/11112222-3333-4444-AAAA-BBBBCCCCDDDD";
        let result = parse_subscription_id_winnow
            .parse(input)
            .map_err(|e| eyre!("Failed to parse subscription id: {e:#}"))?;
        println!("Parsed subscription ID with uppercase UUID: {:?}", result);
        Ok(())
    }

    #[test]
    fn test_mixed_case_uuid() -> eyre::Result<()> {
        let input = "/subscriptions/11112222-3333-4444-AaAa-BbBbCcCcDdDd";
        let result = parse_subscription_id_winnow
            .parse(input)
            .map_err(|e| eyre!("Failed to parse subscription id: {e:#}"))?;
        println!("Parsed subscription ID with mixed case UUID: {:?}", result);
        Ok(())
    }

    #[test]
    fn test_invalid_cases() {
        let invalid_inputs = vec![
            "subscriptions/11112222-3333-4444-aaaa-bbbbccccdddd", // missing leading slash
            "/subscription/11112222-3333-4444-aaaa-bbbbccccdddd", // wrong singular form
            "/subscriptions11112222-3333-4444-aaaa-bbbbccccdddd", // missing slash after subscriptions
            "/subscriptions/",                                    // missing UUID
            "/subscriptions/invalid-uuid",                        // invalid UUID format
            "/subscriptions/11112222-3333-4444-aaaa-bbbbccccdddd-extra", // UUID too long
            "",                                                   // empty string
            "random text",                                        // completely wrong
        ];

        for input in invalid_inputs {
            let result = parse_subscription_id_winnow.parse(input);
            assert!(result.is_err(), "Should have failed to parse: {}", input);
            println!("Correctly rejected: {}", input);
        }
    }

    #[test]
    fn test_random_uuid_roundtrip() -> eyre::Result<()> {
        for _ in 0..5 {
            let random_uuid = uuid::Uuid::new_v4();
            let input = format!("/subscriptions/{}", random_uuid);
            let result = parse_subscription_id_winnow
                .parse(&input)
                .map_err(|e| eyre!("Failed to parse subscription id: {e:#}"))?;
            assert_eq!(*result, random_uuid);
            println!("Successfully roundtripped: {}", input);
        }
        Ok(())
    }

    #[test]
    fn test_with_extra_content() {
        let input = "/subscriptions/11112222-3333-4444-aaaa-bbbbccccdddd/extra/content";
        let mut input_mut = input;
        let result = parse_subscription_id_winnow.parse_next(&mut input_mut);

        // Should successfully parse the subscription ID part
        assert!(result.is_ok());
        // Should leave the rest unparsed
        assert_eq!(input_mut, "/extra/content");
        println!("Parsed with remaining content: {}", input_mut);
    }
}
