use std::time::Duration;

use momento_protos::auth::{auth_client::AuthClient, LoginRequest, LoginResponse};
use thiserror::Error;
use tonic::{codegen::http::uri::InvalidUri, transport::Channel, Streaming};

use crate::{
    endpoint_resolver::MomentoEndpointsResolver,
    response::MomentoError,
    utils::{connect_channel_lazily, ChannelConnectError},
};

pub type EarlyOutActionResult = Option<Result<Credentials, MomentoError>>;
pub type LoginActionConsumer = fn(LoginAction) -> EarlyOutActionResult;

/// Credentials as returned from the auth workflow.
#[derive(Clone, Debug)]
pub struct Credentials {
    token: String,
    valid_for: Duration,
}

impl Credentials {
    pub fn new(token: String, valid_for: Duration) -> Self {
        Self { token, valid_for }
    }

    /// An auth token for use with [`SimpleCacheClient`].
    ///
    /// [`SimpleCacheClient`]: crate::simple_cache_client::SimpleCacheClient
    pub fn token(&self) -> &str {
        &self.token
    }

    /// How long this auth token is valid for.
    pub fn valid_for(&self) -> Duration {
        self.valid_for
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    /// The login process was aborted from the server side.
    #[error("Login was aborted.")]
    LoginAborted,

    /// The login process failed from the server side with an error message.
    #[error("Login failed.")]
    LoginFailed(String),

    /// The server returned an unexpected gRPC status.
    #[error("Error while making a request to the server.")]
    ServerError(#[from] tonic::Status),

    /// The auth URI generated by the SDK was unparseable.
    #[error("Auth URI was invalid.")]
    BadUri(#[source] InvalidUri),

    /// An error occurred while trying to connect to the server.
    #[error("Unable to connect to server.")]
    Connection(#[source] tonic::transport::Error),

    /// The action passed to [`login`] returned an error.
    #[error("The login handler returned an error.")]
    ActionError(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl From<AuthError> for MomentoError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::LoginAborted => MomentoError::Cancelled {
                description: "aborted login".into(),
                source: err.into(),
            },
            AuthError::LoginFailed(_) => MomentoError::PermissionDenied {
                description: "login failed".into(),
                source: err.into(),
            },
            AuthError::BadUri(_) => MomentoError::BadRequest {
                description: "bad uri".into(),
                source: Some(err.into()),
            },
            AuthError::Connection(_) => MomentoError::InternalServerError {
                description: "connection failed".into(),
                source: err.into(),
            },
            AuthError::ServerError(_) => MomentoError::InternalServerError {
                description: "server error".into(),
                source: err.into(),
            },
            AuthError::ActionError(_) => MomentoError::ClientSdkError {
                description: "login action failed".into(),
                source: err.into(),
            },
        }
    }
}

impl From<ChannelConnectError> for AuthError {
    fn from(value: ChannelConnectError) -> Self {
        match value {
            ChannelConnectError::BadUri(err) => Self::BadUri(err),
            ChannelConnectError::Connection(err) => Self::Connection(err),
        }
    }
}

/// Initiate a login workflow.
///
/// You need to provide an implementation for the LoginActions.
///
/// # Example
/// ```rust
/// use momento::auth::{login, LoginAction, AuthError};
///
/// # tokio_test::block_on(async {
/// let result = login(|action| {
///     match action {
///         LoginAction::OpenBrowser(directive) => println!("opening browser to: {}", directive.url),
///         LoginAction::ShowMessage(message) => println!("showing message: {}", message.text),
///     }
///     None // Could instead return an error or an early logged-in or whatever.
///     // Some()
/// }).await;
///
/// match result {
///     Ok(credentials) => println!("Logged in! Session token: {}", credentials.token()),
///     Err(err) => println!("Failed to log in: {err}")
/// }
/// # });
/// ```
pub async fn login(action_sink: LoginActionConsumer) -> Result<Credentials, AuthError> {
    let mut client = auth_client()?;
    let mut stream_response = client.login(LoginRequest {}).await?;

    consume_login_messages(stream_response.get_mut(), action_sink).await
}

/// Things that must be done to move the login process forward
pub enum LoginAction {
    /// You need to open a browser to an interactive login page. The url is in this message.
    OpenBrowser(OpenBrowser),
    /// You can log this; it's for informational purposes.
    ShowMessage(ShowMessage),
}

pub struct ShowMessage {
    pub text: String,
}

pub struct OpenBrowser {
    pub url: String,
}

async fn consume_login_messages(
    stream: &mut Streaming<LoginResponse>,
    action_sink: LoginActionConsumer,
) -> Result<Credentials, AuthError> {
    use momento_protos::auth::login_response::State;

    while let Some(message) = stream.message().await? {
        let early_out_action_result = match message.state {
            Some(state) => match state {
                State::DirectBrowser(direct) => {
                    action_sink(LoginAction::OpenBrowser(OpenBrowser { url: direct.url }))
                }
                State::Message(message) => {
                    action_sink(LoginAction::ShowMessage(ShowMessage { text: message.text }))
                }
                State::LoggedIn(success) => {
                    return Ok(Credentials::new(
                        success.session_token,
                        Duration::from_secs(success.valid_for_seconds.into()),
                    ))
                }
                State::Error(failure) => return Err(AuthError::LoginFailed(failure.description)),
            },
            None => action_sink(LoginAction::ShowMessage(ShowMessage {
                text: "Invalid login state received: no state".to_string(),
            })),
        };

        if let Some(result) = early_out_action_result {
            log::debug!("Early-out of login with result: {:?}", result);
            return result.map_err(|e| AuthError::ActionError(Box::new(e)));
        }
    }

    Err(AuthError::LoginAborted)
}

fn auth_client() -> Result<AuthClient<Channel>, AuthError> {
    let hostname = MomentoEndpointsResolver::get_login_endpoint();
    let channel = connect_channel_lazily(&hostname)?;
    Ok(AuthClient::new(channel))
}
