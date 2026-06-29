use crate::AZURE_DEVOPS_RESOURCE_ID;
use crate::AzureClaims;
use crate::AzureRestResource;
use crate::fetch_azure_access_token;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedJwt {
    pub header_json: String,
    pub claims_json: String,
    pub signature: String,
}

pub async fn get_azure_access_token_jwt() -> eyre::Result<()> {
    let access_token =
        fetch_azure_access_token::<String>(None, AzureRestResource::AzureResourceManager).await?;
    let claims = decode_azure_claims(&access_token.access_token)?;
    eyre::ensure!(
        [
            "https://management.core.windows.net/",
            AZURE_DEVOPS_RESOURCE_ID
        ]
        .contains(&claims.audience.as_str()),
        "unexpected JWT audience: {}",
        claims.audience,
    );
    Ok(())
}

pub fn decode_jwt(token: &str) -> eyre::Result<DecodedJwt> {
    let token = token.trim();
    let mut segments = token.split('.');
    let header = segments
        .next()
        .ok_or_else(|| eyre::eyre!("missing JWT header"))?;
    let claims = segments
        .next()
        .ok_or_else(|| eyre::eyre!("missing JWT claims"))?;
    let signature = segments
        .next()
        .ok_or_else(|| eyre::eyre!("missing JWT signature"))?;
    eyre::ensure!(segments.next().is_none(), "JWT had more than 3 segments");

    Ok(DecodedJwt {
        header_json: decode_jwt_segment(header, "header")?,
        claims_json: decode_jwt_segment(claims, "claims")?,
        signature: signature.to_string(),
    })
}

fn decode_azure_claims(token: &str) -> eyre::Result<AzureClaims> {
    let decoded = decode_jwt(token)?;
    let claims = facet_json::from_str::<AzureClaims>(&decoded.claims_json).map_err(|error| {
        eyre::eyre!(
            "failed to deserialize Azure JWT claims with facet_json: {:?}",
            error
        )
    })?;
    Ok(claims)
}

fn decode_jwt_segment(segment: &str, label: &str) -> eyre::Result<String> {
    let bytes = BASE64_URL_SAFE_NO_PAD
        .decode(segment)
        .map_err(|error| eyre::eyre!("failed to base64url-decode JWT {label}: {error}"))?;
    let json = std::str::from_utf8(&bytes)
        .map_err(|error| eyre::eyre!("JWT {label} was not valid UTF-8: {error}"))?;
    Ok(json.to_string())
}

#[cfg(test)]
mod test {
    use base64::Engine;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;

    #[test]
    fn decodes_claims() -> eyre::Result<()> {
        let header = r#"{"alg":"RS256","typ":"JWT"}"#;
        let claims = r#"{
            "aud": "https://management.core.windows.net/",
            "iss": "https://sts.windows.net/11111111-1111-1111-1111-111111111111/",
            "iat": 1712345678,
            "nbf": 1712345678,
            "exp": 1712349278,
            "acr": "1",
            "acrs": ["p1"],
            "aio": "abc",
            "amr": ["pwd"],
            "appid": "22222222-2222-2222-2222-222222222222",
            "appidacr": "0",
            "deviceid": null,
            "family_name": "Lovelace",
            "given_name": "Ada",
            "groups": ["33333333-3333-3333-3333-333333333333"],
            "idtyp": "user",
            "ipaddr": "127.0.0.1",
            "name": "Ada Lovelace",
            "oid": "44444444-4444-4444-4444-444444444444",
            "puid": "puid",
            "pwd_url": null,
            "rh": "rh",
            "scp": "user_impersonation",
            "sid": "55555555-5555-5555-5555-555555555555",
            "sub": "sub",
            "tid": "11111111-1111-1111-1111-111111111111",
            "unique_name": "ada@example.test",
            "upn": "ada@example.test",
            "uti": "uti",
            "ver": "1.0",
            "wids": ["66666666-6666-6666-6666-666666666666"],
            "xms_ftd": "ftd",
            "xms_idrel": "idrel",
            "xms_tcdt": 1712345678
        }"#;
        let token = format!(
            "{}.{}.{}",
            BASE64_URL_SAFE_NO_PAD.encode(header),
            BASE64_URL_SAFE_NO_PAD.encode(claims),
            BASE64_URL_SAFE_NO_PAD.encode("signature")
        );

        let decoded = super::decode_jwt(&token)?;
        assert_eq!(decoded.header_json, header);
        assert_eq!(decoded.claims_json, claims);
        assert_eq!(
            decoded.signature,
            BASE64_URL_SAFE_NO_PAD.encode("signature")
        );

        let claims = super::decode_azure_claims(&token)?;

        assert_eq!(claims.name, "Ada Lovelace");
        assert_eq!(claims.audience, "https://management.core.windows.net/");
        Ok(())
    }

    #[test]
    fn decodes_jwt_with_surrounding_whitespace() -> eyre::Result<()> {
        let token = format!(
            "{}.{}.{}",
            BASE64_URL_SAFE_NO_PAD.encode("{}"),
            BASE64_URL_SAFE_NO_PAD.encode(r#"{"sub":"hello"}"#),
            "sig"
        );

        let decoded = super::decode_jwt(&format!("  {token}\n"))?;

        assert_eq!(decoded.claims_json, r#"{"sub":"hello"}"#);
        assert_eq!(decoded.signature, "sig");
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        super::get_azure_access_token_jwt().await?;
        Ok(())
    }
}
