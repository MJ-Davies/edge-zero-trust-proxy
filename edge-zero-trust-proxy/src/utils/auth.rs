use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode_header, DecodingKey, Validation, Algorithm, jwk::JwkSet};
use worker::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct AccessClaims {
    pub email: String,
    pub aud: Vec<String>,
    pub exp: usize,
}

pub async fn fetch_jwks(env: &Env) -> Result<JwkSet> {
    let team_domain = env.var("TEAM_DOMAIN")?.to_string();
    let certs_url = format!("{}/cdn-cgi/access/certs", team_domain);
    
    let mut req = Request::new(&certs_url, Method::Get)?;
    let mut res = Fetch::Request(req).send().await?;
    
    let jwks: JwkSet = res.json().await.map_err(|e| Error::RustError(e.to_string()))?;
    Ok(jwks)
}

pub async fn verify_identity(req: &Request, env: &Env) -> Result<String> {
    let jwt = match req.headers().get("Cf-Access-Jwt-Assertion")? {
        Some(token) => token,
        None => return Err(Error::RustError("Unauthorized: Missing CF Access Token".into())),
    };

    let header = decode_header(&jwt).map_err(|e| Error::RustError(e.to_string()))?;
    let kid = header.kid.ok_or_else(|| Error::RustError("Missing kid in JWT".into()))?;

    let jwks = fetch_jwks(env).await?;
    let jwk = jwks.find(&kid).ok_or_else(|| Error::RustError("Key not found in JWKS".into()))?;
    let decoding_key = DecodingKey::from_jwk(jwk).map_err(|e| Error::RustError(e.to_string()))?;

    let aud_tag = env.var("AUD_TAG")?.to_string();
    let team_domain = env.var("TEAM_DOMAIN")?.to_string();
    
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&aud_tag]);
    validation.set_issuer(&[&team_domain]);

    let token_data = jsonwebtoken::decode::<AccessClaims>(
        &jwt,
        &decoding_key,
        &validation,
    ).map_err(|e| Error::RustError(format!("JWT Validation Failed: {}", e)))?;

    Ok(token_data.claims.email)
}
