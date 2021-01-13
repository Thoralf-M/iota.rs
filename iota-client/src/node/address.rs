// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{parse_response, types::Bech32Address, AddressBalance, AddressOutputs, Client, Error, Response, Result};

use bee_message::prelude::{TransactionId, UTXOInput};

use std::convert::TryInto;

/// Builder of GET /api/v1/address/{address} endpoint
pub struct GetAddressBuilder<'a> {
    client: &'a Client,
}

impl<'a> GetAddressBuilder<'a> {
    /// Create GET /api/v1/address/{address} endpoint builder
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Consume the builder and get the balance of a given Bech32 encoded address.
    /// If count equals maxResults, then there might be more outputs available but those were skipped for performance
    /// reasons. User should sweep the address to reduce the amount of outputs.
    pub async fn balance(self, address: &Bech32Address) -> Result<u64> {
        let mut url = self.client.get_node()?;
        url.set_path(&format!("api/v1/addresses/{}", address));
        let resp = reqwest::get(url).await?;

        parse_response!(resp, 200 => {
            let r = resp.json::<Response<AddressBalance>>().await?.data;
            Ok(r.balance)
        })
    }

    /// Consume the builder and get all outputs that use a given address.
    /// If count equals maxResults, then there might be more outputs available but those were skipped for performance
    /// reasons. User should sweep the address to reduce the amount of outputs.
    pub async fn outputs(self, address: &Bech32Address) -> Result<Box<[UTXOInput]>> {
        let mut url = self.client.get_node()?;
        url.set_path(&format!("api/v1/addresses/{}/outputs", address));
        let resp = reqwest::get(url).await?;

        parse_response!(resp, 200 => {
            let r = resp.json::<Response<AddressOutputs>>().await?.data.output_ids;
            r.iter()
                .map(|s| {
                    let mut transaction_id = [0u8; 32];
                    hex::decode_to_slice(&s[..64], &mut transaction_id)?;
                    let index = u16::from_le_bytes(
                        hex::decode(&s[64..]).map_err(|_| Error::InvalidParameter("index".to_string()))?[..]
                            .try_into()
                            .map_err(|_| Error::InvalidParameter("index".to_string()))?,
                    );
                    Ok(UTXOInput::new(TransactionId::new(transaction_id), index)?)
                })
                .collect::<Result<Box<[UTXOInput]>>>()
        })
    }
}
