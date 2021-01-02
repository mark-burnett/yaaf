use ::thiserror::Error;
use ::tokio::sync::{broadcast, mpsc};

#[derive(Debug, Error)]
pub enum YaafError {
    #[error(transparent)]
    AddressError(#[from] AddressError),
    #[error(transparent)]
    ContextError(#[from] ContextError),
    #[error(transparent)]
    SystemError(#[from] SystemError),
}

#[derive(Debug, Error)]
pub enum AddressError {
    #[error("failed to find channel")]
    ChannelLookupError,
    #[error("failed to tell actor")]
    TellFailure { source: YaafInternalError },
}

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("failed to find channel")]
    ChannelLookupError,
    #[error("failed to broadcast message")]
    BroadcastFailure { source: YaafInternalError },
}

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("failed to add actor")]
    AddActorFailure { source: YaafInternalError },
    #[error("failed to add source")]
    AddSourceFailure { source: YaafInternalError },
    #[error("failed to create system")]
    CreateError { source: YaafInternalError },
    #[error("failed to add actor")]
    ShutdownError { source: YaafInternalError },
}

#[derive(Debug, Error)]
pub enum YaafInternalError {
    #[error("failed to create channel")]
    CreateChannelFailure { source: Box<YaafInternalError> },
    #[error("failed to subscribe to messages")]
    MailboxSubscribeFailure { source: Box<YaafInternalError> },
    #[error("failed to subscribe to system events")]
    MailboxSystemSubscribeFailure { source: Box<YaafInternalError> },
    #[error("failed to receive subscription confirmation")]
    ReceiveFailure,
    #[error("failed to lookup channel")]
    ChannelLookupFailure,
    #[error("failed to send")]
    SendFailure,
}

impl<T> From<broadcast::error::SendError<T>> for YaafInternalError {
    fn from(_src: broadcast::error::SendError<T>) -> Self {
        YaafInternalError::SendFailure
    }
}

impl<T> From<mpsc::error::SendError<T>> for YaafInternalError {
    fn from(_src: mpsc::error::SendError<T>) -> Self {
        YaafInternalError::SendFailure
    }
}

impl From<mpsc::error::RecvError> for YaafInternalError {
    fn from(_src: mpsc::error::RecvError) -> Self {
        YaafInternalError::ReceiveFailure
    }
}
