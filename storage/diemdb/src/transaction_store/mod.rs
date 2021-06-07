// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines transaction store APIs that are related to committed signed transactions.

use crate::{
    change_set::ChangeSet,
    errors::DiemDbError,
    schema::{transaction::TransactionSchema, transaction_by_account::TransactionByAccountSchema},
};
use anyhow::{ensure, format_err, Result};
use diem_types::{
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    transaction::{Transaction, Version},
};
use schemadb::{ReadOptions, SchemaIterator, DB};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct TransactionStore {
    db: Arc<DB>,
}

impl TransactionStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    /// Gets the version of a transaction by the sender `address` and `sequence_number`.
    pub fn get_account_transaction_version(
        &self,
        address: AccountAddress,
        sequence_number: u64,
        ledger_version: Version,
    ) -> Result<Option<Version>> {
        if let Some(version) = self
            .db
            .get::<TransactionByAccountSchema>(&(address, sequence_number))?
        {
            if version <= ledger_version {
                return Ok(Some(version));
            }
        }

        Ok(None)
    }

    /// Gets an iterator that yields `(sequence_number, version)` for each
    /// transaction sent by an account, starting at `start_seq_num`, and returning
    /// at most `num_versions` results with `version <= ledger_version`
    pub fn get_account_transaction_version_iter(
        &self,
        address: AccountAddress,
        start_seq_num: u64,
        num_versions: u64,
        ledger_version: Version,
    ) -> Result<AccountTransactionVersionIter> {
        let mut iter = self
            .db
            .iter::<TransactionByAccountSchema>(ReadOptions::default())?;
        iter.seek(&(address, start_seq_num))?;
        Ok(AccountTransactionVersionIter {
            inner: iter,
            address,
            expected_next_seq_num: start_seq_num,
            end_seq_num: start_seq_num
                .checked_add(num_versions)
                .ok_or_else(|| format_err!("too many transactions requested"))?,
            prev_version: None,
            ledger_version,
        })
    }

    /// Get signed transaction given `version`
    pub fn get_transaction(&self, version: Version) -> Result<Transaction> {
        self.db
            .get::<TransactionSchema>(&version)?
            .ok_or_else(|| DiemDbError::NotFound(format!("Txn {}", version)).into())
    }

    /// Gets an iterator that yields `num_transactions` transactions starting from `start_version`.
    pub fn get_transaction_iter(
        &self,
        start_version: Version,
        num_transactions: usize,
    ) -> Result<TransactionIter> {
        let mut iter = self.db.iter::<TransactionSchema>(ReadOptions::default())?;
        iter.seek(&start_version)?;
        Ok(TransactionIter {
            inner: iter,
            expected_next_version: start_version,
            end_version: start_version
                .checked_add(num_transactions as u64)
                .ok_or_else(|| format_err!("too many transactions requested"))?,
        })
    }

    /// Returns the block metadata carried on the block metadata transaction at or preceding
    /// `version`, together with the version of the block metadata transaction.
    /// Returns None if there's no such transaction at or preceding `version` (it's likely the genesis
    /// version 0).
    pub fn get_block_metadata(&self, version: Version) -> Result<Option<(Version, BlockMetadata)>> {
        // Maximum TPS from benchmark is around 1000.
        const MAX_VERSIONS_TO_SEARCH: usize = 1000 * 3;

        // Linear search via `DB::rev_iter()` here, NOT expecting performance hit, due to the fact
        // that the iterator caches data block and that there are limited number of transactions in
        // each block.
        let mut iter = self.db.rev_iter::<TransactionSchema>(Default::default())?;
        iter.seek(&version)?;
        for res in iter.take(MAX_VERSIONS_TO_SEARCH) {
            let (v, txn) = res?;
            if let Transaction::BlockMetadata(block_meta) = txn {
                return Ok(Some((v, block_meta)));
            } else if v == 0 {
                return Ok(None);
            }
        }

        Err(DiemDbError::NotFound(format!("BlockMetadata preceding version {}", version)).into())
    }

    /// Save signed transaction at `version`
    pub fn put_transaction(
        &self,
        version: Version,
        transaction: &Transaction,
        cs: &mut ChangeSet,
    ) -> Result<()> {
        if let Transaction::UserTransaction(txn) = transaction {
            cs.batch.put::<TransactionByAccountSchema>(
                &(txn.sender(), txn.sequence_number()),
                &version,
            )?;
        }
        cs.batch.put::<TransactionSchema>(&version, &transaction)?;

        Ok(())
    }
}

pub struct TransactionIter<'a> {
    inner: SchemaIterator<'a, TransactionSchema>,
    expected_next_version: Version,
    end_version: Version,
}

impl<'a> TransactionIter<'a> {
    fn next_impl(&mut self) -> Result<Option<Transaction>> {
        if self.expected_next_version >= self.end_version {
            return Ok(None);
        }

        let ret = match self.inner.next().transpose()? {
            Some((version, transaction)) => {
                ensure!(
                    version == self.expected_next_version,
                    "Transaction versions are not consecutive.",
                );
                self.expected_next_version += 1;
                Some(transaction)
            }
            None => None,
        };

        Ok(ret)
    }
}

impl<'a> Iterator for TransactionIter<'a> {
    type Item = Result<Transaction>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().transpose()
    }
}

pub struct AccountTransactionVersionIter<'a> {
    inner: SchemaIterator<'a, TransactionByAccountSchema>,
    address: AccountAddress,
    expected_next_seq_num: u64,
    end_seq_num: u64,
    prev_version: Option<Version>,
    ledger_version: Version,
}

impl<'a> AccountTransactionVersionIter<'a> {
    fn next_impl(&mut self) -> Result<Option<(u64, Version)>> {
        if self.expected_next_seq_num >= self.end_seq_num {
            return Ok(None);
        }

        Ok(match self.inner.next().transpose()? {
            Some(((address, seq_num), version)) => {
                // TODO(philiphayes): what guarantees do we make about sequence
                // numbers and transactions for one account? In the current documentation,
                // we guarantee that sequence numbers are contiguous (as below),
                // but is this required in the future? what about proposals for
                // non-contiguous sequence numbers, how does that fit in here or
                // is that at a higher layer?

                // No more transactions sent by this account.
                if address != self.address {
                    return Ok(None);
                }

                // Ensure seq_num_{i+1} == seq_num_{i} + 1
                ensure!(
                    seq_num == self.expected_next_seq_num,
                    "DB corruption: account transactions sequence numbers are not contiguous: \
                     actual: {}, expected: {}",
                    seq_num,
                    self.expected_next_seq_num,
                );

                // Ensure version_{i+1} > version_{i}
                if let Some(prev_version) = self.prev_version {
                    ensure!(
                        prev_version < version,
                        "DB corruption: account transaction versions are not strictly increasing: \
                         previous version: {}, current version: {}",
                        prev_version,
                        version,
                    );
                }

                // No more transactions (in this view of the ledger).
                if version > self.ledger_version {
                    return Ok(None);
                }

                self.expected_next_seq_num += 1;
                self.prev_version = Some(version);
                Some((seq_num, version))
            }
            None => None,
        })
    }
}

impl<'a> Iterator for AccountTransactionVersionIter<'a> {
    type Item = Result<(u64, Version)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().transpose()
    }
}

#[cfg(test)]
mod test;
