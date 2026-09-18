use crate::{options::CallbackArg1, AuthError};

/// Authentication configuration used when connecting to the NATS server.
///
/// This struct holds the various credentials that can be supplied to
/// authenticate a client connection. Depending on the server configuration,
/// only some of these fields will be relevant, and some combinations are
/// mutually exclusive (for example, using `jwt`/`signature_callback` together
/// is a different authentication mechanism than plain `username`/`password`
/// or a bare `token`).
#[derive(Default)]
pub struct Auth {
    /// A user JWT used for JWT-based authentication. When set, it is
    /// typically paired with `signature_callback` (or `signature`), which
    /// is used to sign the server-provided nonce with the corresponding
    /// NKEY to prove ownership of the JWT.
    pub jwt: Option<String>,
    /// An NKEY (public key) used for NKEY-based authentication, where the
    /// server issues a nonce that must be signed with the matching seed.
    pub nkey: Option<String>,
    /// Callback invoked with a server-provided nonce to produce a signature,
    /// used in conjunction with `jwt` for JWT-based authentication. This is
    /// crate-private and set internally when a signing callback is
    /// configured on the connection options.
    pub(crate) signature_callback: Option<CallbackArg1<String, Result<String, AuthError>>>,
    /// A pre-computed signature bytes, used together with `jwt` for
    /// JWT-based authentication when a `signature_callback` is not used.
    pub signature: Option<Vec<u8>>,
    /// Username for basic username/password authentication.
    pub username: Option<String>,
    /// Password for basic username/password authentication, used together
    /// with `username`.
    pub password: Option<String>,
    /// A bare authentication token, used as an alternative to
    /// username/password or JWT/NKEY based authentication.
    pub token: Option<String>,
}

impl Auth {
    pub fn new() -> Auth {
        Auth::default()
    }
}
