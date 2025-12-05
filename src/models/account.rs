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

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use super::*;

    #[test]
    fn deposit_locked_account_fails() {
        let mut account = Account::new(1);
        account.locked = true;

        let result = account.deposit(dec!(10.0));

        assert!(result.is_err());
        assert!(account.available == Decimal::ZERO);
    }

    #[test]
    fn deposit_negative_amount_fails() {
        let mut account = Account::new(1);
        account.deposit(dec!(100.0)).unwrap();

        let result = account.deposit(dec!(-0.001));

        assert!(result.is_err());
        assert!(account.available == dec!(100.0));
    }

    #[test]
    fn deposit_increases_available_and_total() {
        let mut account = Account::new(1);

        account.deposit(dec!(100.01)).unwrap();

        assert_eq!(account.available, dec!(100.01));
        assert_eq!(account.total, dec!(100.01));
    }

    #[test]
    fn withdraw_locked_account_fails() {
        let mut account = Account::new(1);
        account.deposit(dec!(100.0)).unwrap();
        account.locked = true;

        let result = account.withdraw(dec!(10.0));

        assert!(result.is_err());
        assert!(account.available == dec!(100.0));
    }

    #[test]
    fn withdraw_negative_amount_fails() {
        let mut account = Account::new(1);
        account.deposit(dec!(100.0)).unwrap();

        let result = account.withdraw(dec!(-0.001));

        assert!(result.is_err());
        assert!(account.available == dec!(100.0));
    }

    #[test]
    fn withdraw_decreases_available_and_total() {
        let mut account = Account::new(1);
        account.deposit(dec!(100.01)).unwrap();

        account.withdraw(dec!(50.00)).unwrap();

        assert_eq!(account.available, dec!(50.01));
        assert_eq!(account.total, dec!(50.01));
    }
}
