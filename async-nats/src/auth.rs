use crate::{options::CallbackArg1, AuthError};

/// Authentication configuration used to authenticate with the NATS server.
///
/// This can include a JWT, an NKey, a signature callback, or simple
/// username/password or token credentials, depending on how the server is
/// configured to authenticate clients.
#[derive(Default)]
pub struct Auth {
    /// A JWT used for authentication with the server.
    pub jwt: Option<String>,
    /// An NKey used for authentication with the server.
    pub nkey: Option<String>,
    pub(crate) signature_callback: Option<CallbackArg1<String, Result<String, AuthError>>>,
    /// A signature, typically produced from signing a server-provided nonce
    /// with an NKey, used for authentication with the server.
    pub signature: Option<Vec<u8>>,
    /// The username used for basic username/password authentication.
    pub username: Option<String>,
    /// The password used for basic username/password authentication.
    pub password: Option<String>,
    /// A token used for token-based authentication.
    pub token: Option<String>,
}

impl Auth {
    /// Creates a new, empty `Auth` with all fields set to `None`.
    pub fn new() -> Auth {
        Auth::default()
    }
}
