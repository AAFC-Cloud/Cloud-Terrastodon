use cloud_terrastodon_credentials::decode_jwt;
use eyre::Result;
use facet_value::Value;
use std::io::Read;

/// Decode a JWT and print its header and claims as JSON.
#[derive(facet::Facet, Debug, Clone)]
pub struct JwtDecodeArgs {
    /// JWT string or '-' to read from stdin
    #[facet(figue::positional)]
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Eq, facet::Facet)]
struct DecodedJwtOutput {
    header: Value,
    claims: Value,
    signature: String,
}

impl JwtDecodeArgs {
    pub async fn invoke(self) -> Result<()> {
        let token = read_token(&self.input)?;
        let decoded = decode_token_to_output(&token)?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &decoded)?;
        println!();
        Ok(())
    }
}

fn read_token(input: &str) -> Result<String> {
    let token = if input == "-" {
        let mut stdin_buf = String::new();
        std::io::stdin().read_to_string(&mut stdin_buf)?;
        stdin_buf
    } else {
        input.to_string()
    };

    let token = token.trim().to_string();
    eyre::ensure!(!token.is_empty(), "JWT input was empty");
    Ok(token)
}

fn decode_token_to_output(token: &str) -> Result<DecodedJwtOutput> {
    let decoded = decode_jwt(token)?;
    Ok(DecodedJwtOutput {
        header: facet_json::from_str(&decoded.header_json)?,
        claims: facet_json::from_str(&decoded.claims_json)?,
        signature: decoded.signature,
    })
}

#[cfg(test)]
mod tests {
    use super::decode_token_to_output;
    use base64::Engine;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;

    #[test]
    fn decodes_header_claims_and_signature() -> eyre::Result<()> {
        let token = format!(
            "{}.{}.{}",
            BASE64_URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#),
            BASE64_URL_SAFE_NO_PAD.encode(r#"{"sub":"alice","roles":["reader"]}"#),
            "signature-segment"
        );

        let decoded = decode_token_to_output(&token)?;

        assert_eq!(
            facet_json::to_string(&decoded.header)?,
            r#"{"alg":"none","typ":"JWT"}"#
        );
        assert_eq!(
            facet_json::to_string(&decoded.claims)?,
            r#"{"sub":"alice","roles":["reader"]}"#
        );
        assert_eq!(decoded.signature, "signature-segment");
        Ok(())
    }
}
