use crate::keyboard::KeyboardEvent;
use std::{env, io};
use thiserror::Error;
use tokio::sync::watch::error;
use toml::de;

/// Result type alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed at parsing packages with alpm.")]
    PackageParsing(#[from] alpm::Error),

    #[error("IO error.")]
    IoError(#[from] io::Error),

    #[error("Failed loading configuration.")]
    PathError(#[from] env::VarError),

    #[error("Failed loading user configuration.")]
    UserConfigurationError(#[from] de::Error),

    #[error("Failed loading pacman.conf.")]
    PacmanConfError(#[from] pacmanconf::Error),

    #[error("Failed to access super-user rights.")]
    SuperUserError,

    #[error("Failed to send event between tasks")]
    EventSendError(#[from] error::SendError<KeyboardEvent>),

    #[error("Failed to receive event between tasks")]
    EventReceiveError(#[from] error::RecvError),
}
