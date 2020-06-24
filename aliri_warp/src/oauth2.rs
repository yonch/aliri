//! Warp filters for validating JWTs against OAuth2 authorities and scopes

use aliri::Authority;
use aliri_jose::{jwt, Jwt};
use aliri_oauth2::{
    jwks::{AuthorityError, RemoteAuthority},
    HasScopes, ScopesPolicy,
};
use serde::Deserialize;
use thiserror::Error;
use warp::Filter;

/// An error during JWT verification
#[derive(Debug, Error)]
#[error("error verifying jwt")]
pub struct AuthFailed(#[from] AuthorityError);

impl warp::reject::Reject for AuthFailed {}

async fn check_jwt<C: for<'de> Deserialize<'de> + HasScopes>(
    jwt: Jwt,
    authority: &RemoteAuthority,
    policy: &ScopesPolicy,
) -> Result<jwt::Claims<C>, AuthFailed> {
    let c: jwt::Claims<C> = authority.verify(&jwt, &policy).await?;
    Ok(c)
}

/// Require the JWT to be valid according to the JWKS authority and scope
/// scopesets
pub fn require_scopes<C, F, A, P>(
    jwt: F,
    authority: A,
    policy: P,
) -> impl Filter<Extract = (jwt::Claims<C>,), Error = warp::Rejection> + Clone
where
    C: for<'de> Deserialize<'de> + HasScopes,
    F: Filter<Extract = (Jwt,), Error = warp::Rejection> + Clone,
    A: AsRef<RemoteAuthority> + Clone + Send + Sync + 'static,
    P: AsRef<ScopesPolicy> + Clone + Send + Sync + 'static,
{
    jwt.and_then(move |jwt: Jwt| {
        let authority = authority.clone();
        let policy = policy.clone();
        async move {
            check_jwt(jwt, authority.as_ref(), policy.as_ref())
                .await
                .map_err(warp::reject::custom)
        }
    })
}