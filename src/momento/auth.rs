use momento_protos::auth::{auth_client::AuthClient, LoginRequest, LoginResponse};
use tonic::{transport::Channel, Streaming};

use crate::{
    endpoint_resolver::MomentoEndpointsResolver, response::error::MomentoError,
    utils::connect_channel_lazily,
};

/// Initiate a login workflow
/// You need to provide an implementation for the LoginActions.
/// ```rust
/// # use momento::momento::auth::{login, LoginAction, LoginResult};
/// # async {
///     let result = login(|action| {
///       match action {
///         LoginAction::OpenBrowser(directive) => println!("opening browser to: {}", directive.url),
///         LoginAction::ShowMessage(message) => println!("showing message: {}", message.text),
///       }
///     });
///     match result.await {
///       LoginResult::LoggedIn(logged_in) => println!("logged in. session token: {}", logged_in.session_token),
///       LoginResult::NotLoggedIn(not_logged_in) => println!("failed to log in: {}", not_logged_in.error_message),
///     };
/// # };
/// ```
pub async fn login<F>(action_sink: F) -> LoginResult
where
    F: Fn(LoginAction),
{
    let mut client = match auth_client() {
        Ok(client) => client,
        Err(error) => return not_logged_in(format!("Failed to create a channel: {:?}", error)),
    };

    let mut stream_response = match client.login(LoginRequest {}).await {
        Ok(response) => response,
        Err(error) => return not_logged_in(format!("Could not get a login response: {:?}", error)),
    };

    let stream = stream_response.get_mut();

    match consume_login_messages(stream, &action_sink).await {
        Ok(result) => result,
        Err(error) => not_logged_in(format!("Failed to log in: {:?}", error)),
    }
}

pub enum LoginResult {
    LoggedIn(LoggedIn),
    NotLoggedIn(NotLoggedIn),
}

/// Things that must be done to move the login process forward
pub enum LoginAction {
    /// You need to open a browser to an interactive login page. The url is in this message.
    OpenBrowser(OpenBrowser),
    /// You can log this; it's for informational purposes.
    ShowMessage(ShowMessage),
}

pub struct LoggedIn {
    pub session_token: String,
}

pub struct NotLoggedIn {
    pub error_message: String,
}

pub struct ShowMessage {
    pub text: String,
}

pub struct OpenBrowser {
    pub url: String,
}

fn not_logged_in(message: String) -> LoginResult {
    LoginResult::NotLoggedIn(NotLoggedIn {
        error_message: message,
    })
}

async fn consume_login_messages<F>(
    stream: &mut Streaming<LoginResponse>,
    action_sink: &F,
) -> Result<LoginResult, MomentoError>
where
    F: Fn(LoginAction),
{
    while let Some(message) = stream.message().await? {
        match message.state {
            Some(state) => match state {
                momento_protos::auth::login_response::State::DirectBrowser(direct) => {
                    action_sink(LoginAction::OpenBrowser(OpenBrowser { url: direct.url }))
                }
                momento_protos::auth::login_response::State::Message(message) => {
                    action_sink(LoginAction::ShowMessage(ShowMessage { text: message.text }))
                }
                momento_protos::auth::login_response::State::LoggedIn(success) => {
                    return Ok(LoginResult::LoggedIn(LoggedIn {
                        session_token: success.session_token,
                    }))
                }
                momento_protos::auth::login_response::State::Error(failure) => {
                    return Ok(LoginResult::NotLoggedIn(NotLoggedIn {
                        error_message: failure.description,
                    }))
                }
            },
            None => action_sink(LoginAction::ShowMessage(ShowMessage {
                text: "Invalid login state received: no state".to_string(),
            })),
        }
    }

    Ok(LoginResult::NotLoggedIn(NotLoggedIn {
        error_message: "Login was aborted".to_string(),
    }))
}

fn auth_client() -> Result<AuthClient<Channel>, MomentoError> {
    let hostname = MomentoEndpointsResolver::get_login_endpoint();
    let channel = connect_channel_lazily(&hostname)?;
    Ok(AuthClient::new(channel))
}
