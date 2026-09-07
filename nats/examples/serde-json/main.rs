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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    first_name: String,
    last_name: String,
    age: u8,
}

fn main() -> std::io::Result<()> {
    let nc = nats::connect("demo.nats.io")?;
    let subj = nc.new_inbox();

    let p = Person {
        first_name: "derek".to_owned(),
        last_name: "collison".to_owned(),
        age: 22,
    };

    let sub = nc.subscribe(&subj)?;
    nc.publish(&subj, serde_json::to_vec(&p)?)?;

    let mut p2 = sub.iter().map(move |msg| {
        let p: Person = serde_json::from_slice(&msg.data).unwrap();
        p
    });
    println!("received {:?}", p2.next().unwrap());

    Ok(())
}
