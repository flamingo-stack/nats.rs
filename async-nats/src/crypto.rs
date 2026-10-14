// Copyright 2020-2024 The NATS Authors
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

#[cfg(feature = "aws-lc-rs")]
use aws_lc_rs as crypto_backend;
#[cfg(not(feature = "aws-lc-rs"))]
use ring as crypto_backend;

#[cfg(not(any(feature = "aws-lc-rs", feature = "ring")))]
compile_error!("Please enable the `aws-lc-rs` or `ring` feature");

use crypto_backend::digest::{Context, SHA256};

pub(crate) struct Sha256(Context);

impl Sha256 {
    pub(crate) fn new() -> Self {
        Self(Context::new(&SHA256))
    }

    pub(crate) fn update(&mut self, chunk: &[u8]) {
        self.0.update(chunk);
    }

    pub(crate) fn finish(self) -> [u8; 32] {
        let digest = self.0.finish();
        digest.as_ref().try_into().expect("sha256 hash is 32 bytes")
    }
}
