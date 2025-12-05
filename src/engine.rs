use anyhow::Result;
use std::collections::HashMap;

use crate::models::{account::Account, transaction::Transaction};

pub struct Engine {
    accounts: HashMap<u16, Account>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            accounts: HashMap::new(),
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<()> {
        let client_id = transaction.client();

        let account = self
            .accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));

        account.process_transaction(transaction)
    }

    pub fn get_accounts(&self) -> &HashMap<u16, Account> {
        &self.accounts
    }
}
