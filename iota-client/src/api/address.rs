// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Client, Error, Result};

use bee_message::prelude::{Address, Ed25519Address};
use bee_signing_ext::{
    binary::{BIP32Path, Ed25519PrivateKey, Ed25519Seed},
    Seed,
};
use blake2::{
    digest::{Update, VariableOutput},
    VarBlake2b,
};
use core::convert::TryInto;
use std::ops::Range;

const HARDEND: u32 = 1 << 31;

/// Builder of find_addresses API
pub struct GetAddressesBuilder<'a> {
    _client: &'a Client,
    seed: &'a Seed,
    account_index: Option<usize>,
    range: Option<Range<usize>>,
}

impl<'a> GetAddressesBuilder<'a> {
    /// Create find_addresses builder
    pub fn new(_client: &'a Client, seed: &'a Seed) -> Self {
        Self {
            _client,
            seed,
            account_index: None,
            range: None,
        }
    }

    /// Sets the account index.
    pub fn account_index(mut self, account_index: usize) -> Self {
        self.account_index = Some(account_index);
        self
    }

    /// Set range to the builder
    pub fn range(mut self, range: Range<usize>) -> Self {
        self.range = Some(range);
        self
    }

    /// Consume the builder and get the vector of Address
    pub fn get(self) -> Result<Vec<(Address, bool)>> {
        let mut path = self
            .account_index
            .map(|i| BIP32Path::from_str(&crate::account_path!(i)).expect("invalid account index"))
            .ok_or_else(|| Error::MissingParameter(String::from("account index")))?;

        let range = match self.range {
            Some(r) => r,
            None => 0..20,
        };

        let seed = match self.seed {
            Seed::Ed25519(s) => s,
            _ => panic!("Other seed scheme isn't supported yet."),
        };

        let mut addresses = Vec::new();
        for i in range {
            let address = generate_address(&seed, &mut path, i, false);
            let internal_address = generate_address(&seed, &mut path, i, true);
            addresses.push((address, false));
            addresses.push((internal_address, true));
        }

        Ok(addresses)
    }
}

fn generate_address(seed: &Ed25519Seed, path: &mut BIP32Path, index: usize, internal: bool) -> Address {
    path.push(internal as u32 + HARDEND);
    path.push(index as u32 + HARDEND);

    let public_key = Ed25519PrivateKey::generate_from_seed(seed, &path)
        .expect("Invalid Seed & BIP32Path. Probably because the index of path is not hardened.")
        .generate_public_key()
        .to_bytes();
    // Hash the public key to get the address
    let mut hasher = VarBlake2b::new(32).unwrap();
    hasher.update(public_key);
    let mut result: [u8; 32] = [0; 32];
    hasher.finalize_variable(|res| {
        result = res.try_into().expect("Invalid Length of Public Key");
    });

    path.pop();
    path.pop();

    Address::Ed25519(Ed25519Address::new(result))
}
