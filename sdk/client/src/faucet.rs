// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BlockingClient, Error, Result, WaitForTransactionError};
use diem_types::transaction::{authenticator::AuthenticationKey, SignedTransaction};

pub struct FaucetClient {
    url: String,
    json_rpc_client: BlockingClient,
}

impl FaucetClient {
    pub fn new(url: String, json_rpc_url: String) -> Self {
        Self {
            url,
            json_rpc_client: BlockingClient::new(json_rpc_url),
        }
    }

    pub fn fund(
        &self,
        currency_code: &str,
        auth_key: AuthenticationKey,
        amount: u64,
    ) -> Result<()> {
        let client = reqwest::blocking::Client::new();

        let url = format!(
            "{}?amount={}&auth_key={}&currency_code={}&return_txns=true",
            self.url, amount, auth_key, currency_code,
        );

        //TODO Fix faucet
        // The current implementation of Faucet is racy, in that 2 independent requests can come in
        // and result in faucet sending 2 txns to the network with the same sequence number and
        // they race to complete. One request will complete while the other will get a HashMismatch
        // Error.
        //
        // Faucet returns a list of txns that need to be waited on before returning. 1) a txn for
        // creating the account (if it doesn't already exist) and 2) to issue funds to said
        // account.
        'faucet: loop {
            let response = client.post(&url).send().map_err(Error::request)?;
            let status_code = response.status();
            let body = response.text().map_err(Error::decode)?;
            if !status_code.is_success() {
                return Err(Error::status(status_code.as_u16()));
            }
            let bytes = hex::decode(body).map_err(Error::decode)?;
            let txns: Vec<SignedTransaction> = bcs::from_bytes(&bytes).unwrap();

            for txn in txns {
                match self
                    .json_rpc_client
                    .wait_for_signed_transaction(&txn, None, None)
                {
                    Ok(_) => {}
                    Err(WaitForTransactionError::TransactionHashMismatchError(_))
                    | Err(WaitForTransactionError::TransactionExpired) => continue 'faucet,
                    Err(e) => return Err(Error::unknown(e)),
                }
            }

            break;
        }

        Ok(())
    }
}
