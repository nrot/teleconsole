use grammers_client::{
    client::updates::{AuthorizationError, InvocationError},
    types::LoginToken,
    Client, Config,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TgErrors {
    #[error("Authorization error")]
    Auth(#[from] AuthorizationError),
    #[error("IncocationError")]
    Invocation(#[from] InvocationError),
}

pub async fn request_login(
    phone: String,
    config: Config,
) -> Result<(Client, LoginToken), TgErrors> {
    let api_id = config.api_id;
    let api_hash = config.api_hash.clone();
    let mut client = Client::connect(config).await?;
    Ok((
        client.clone(),
        client
            .request_login_code(phone.as_str(), api_id, api_hash.as_str())
            .await?,
    ))
}
