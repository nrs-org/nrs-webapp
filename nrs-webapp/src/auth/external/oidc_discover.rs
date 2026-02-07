use super::Result;
use openidconnect::{
    AdditionalProviderMetadata, AuthDisplay, ClaimName, ClaimType, ClientAuthMethod, GrantType,
    IssuerUrl, JsonWebKey, JweContentEncryptionAlgorithm, JweKeyManagementAlgorithm,
    JwsSigningAlgorithm, ProviderMetadata, ResponseMode, ResponseType, SubjectIdentifierType,
};

use crate::model::ModelManager;

pub async fn oidc_discover<A, AD, CA, CN, CT, G, JE, JK, K, RM, RT, S>(
    mm: &ModelManager,
    issuer_url: &'static str,
) -> Result<ProviderMetadata<A, AD, CA, CN, CT, G, JE, JK, K, RM, RT, S>>
where
    A: AdditionalProviderMetadata,
    AD: AuthDisplay,
    CA: ClientAuthMethod,
    CN: ClaimName,
    CT: ClaimType,
    G: GrantType,
    JE: JweContentEncryptionAlgorithm<
        KeyType = <K::SigningAlgorithm as JwsSigningAlgorithm>::KeyType,
    >,
    JK: JweKeyManagementAlgorithm,
    K: JsonWebKey,
    RM: ResponseMode,
    RT: ResponseType,
    S: SubjectIdentifierType,
{
    let issuer_url = IssuerUrl::new(issuer_url.into()).expect("valid issuer URL");
    let provider_metadata =
        ProviderMetadata::discover_async(issuer_url, mm.http_client_wrapper()).await?;
    Ok(provider_metadata)
}
