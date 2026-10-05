// Copyright 2020-2022 The NATS Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Error kind describing a service request error payload.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// The service returned an error response with the given status and code.
    Request,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Request => write!(f, "service request error"),
        }
    }
}

/// Error returned when a service request fails.
pub type Error = crate::error::Error<ErrorKind>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ErrorPayload {
    pub status: String,
    pub code: usize,
}

impl Display for ErrorPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "service request error code: {}, status: {}",
            self.status, self.code
        )
    }
}

impl std::error::Error for ErrorPayload {}

impl From<ErrorPayload> for Error {
    fn from(payload: ErrorPayload) -> Self {
        Error::with_source(ErrorKind::Request, payload.to_string(), payload)
    }
}
