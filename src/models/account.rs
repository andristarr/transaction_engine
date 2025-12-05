use std::collections::HashMap;

use anyhow::{Result, anyhow, bail};
use rust_decimal::Decimal;

use crate::models::transaction::{Transaction, TransactionType};

pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub withheld: Decimal,
    pub total: Decimal,
    pub locked: bool,
    pub effect_transactions: HashMap<u32, Transaction>,
    pub dispute_transactions: HashMap<u32, Transaction>,
}

impl Account {
    pub fn new(client: u16) -> Self {
        Account {
            client,
            available: Decimal::ZERO,
            withheld: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
            effect_transactions: HashMap::new(),
            dispute_transactions: HashMap::new(),
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<()> {
        if transaction.client() != self.client {
            bail!("Transaction client ID does not match account client ID");
        }

        transaction.run(self)?;

        match transaction.transaction_type() {
            TransactionType::Deposit { .. } | TransactionType::Withdrawal { .. } => {
                self.effect_transactions
                    .insert(transaction.tx(), transaction);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn deposit(&mut self, amount: Decimal) -> Result<()> {
        if self.locked {
            bail!("Account locked");
        }

        if amount < Decimal::ZERO {
            bail!("Deposit amount should be positive");
        }

        self.available += amount;
        self.total += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: Decimal) -> Result<()> {
        if self.locked {
            bail!("Account locked");
        }

        if amount < Decimal::ZERO {
            bail!("Withdrawed amount should be positive");
        }

        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
            Ok(())
        } else {
            bail!("Insufficient funds")
        }
    }

    pub fn dispute(&mut self, tx: &Transaction) -> Result<()> {
        if self.dispute_transactions.contains_key(&tx.tx()) {
            bail!("Transaction already disputed");
        }

        let transaction = self
            .effect_transactions
            .remove(&tx.tx())
            .ok_or_else(|| anyhow!("Transaction not found"))?;

        let amount = transaction.get_amount();

        if self.available < amount {
            bail!("Insufficient available funds to dispute");
        }

        if !matches!(
            transaction.transaction_type(),
            TransactionType::Deposit { .. }
        ) {
            bail!("Only deposit transactions can be disputed");
        }

        self.available -= amount;
        self.withheld += amount;
        self.dispute_transactions.insert(tx.tx(), transaction);
        Ok(())
    }

    pub fn resolve(&mut self, tx: &Transaction) -> Result<()> {
        let transaction = self
            .dispute_transactions
            .remove(&tx.tx())
            .ok_or_else(|| anyhow!("Transaction not disputed"))?;

        let amount = transaction.get_amount();

        self.withheld -= amount;
        self.available += amount;
        self.effect_transactions.insert(tx.tx(), transaction);
        Ok(())
    }

    pub fn chargeback(&mut self, tx: &Transaction) -> Result<()> {
        let transaction = self
            .dispute_transactions
            .remove(&tx.tx())
            .ok_or_else(|| anyhow!("Transaction not disputed"))?;

        let amount = transaction.get_amount();

        self.withheld -= amount;
        self.total -= amount;
        self.locked = true;
        Ok(())
    }
}
