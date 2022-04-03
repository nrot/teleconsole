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
