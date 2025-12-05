use std::collections::HashMap;

use anyhow::{Result, anyhow, bail};
use rust_decimal::Decimal;

use crate::models::transaction::{Transaction, TransactionType};

pub struct Account {
    client: u16,
    available: Decimal,
    withheld: Decimal,
    total: Decimal,
    locked: bool,
    effect_transactions: HashMap<u32, Transaction>,
    dispute_transactions: HashMap<u32, Transaction>,
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

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn available(&self) -> Decimal {
        self.available
    }

    pub fn withheld(&self) -> Decimal {
        self.withheld
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<()> {
        if transaction.client() != self.client {
            bail!("Transaction client ID does not match account client ID");
        }

        match transaction.transaction_type() {
            TransactionType::Deposit { amount } => self.deposit(*amount)?,
            TransactionType::Withdrawal { amount } => self.withdraw(*amount)?,
            TransactionType::Dispute => self.dispute(&transaction)?,
            TransactionType::Resolve => self.resolve(&transaction)?,
            TransactionType::Chargeback => self.chargeback(&transaction)?,
        }

        match transaction.transaction_type() {
            TransactionType::Deposit { .. } | TransactionType::Withdrawal { .. } => {
                self.effect_transactions
                    .insert(transaction.tx(), transaction);
            }
            _ => {}
        }
        Ok(())
    }

    fn deposit(&mut self, amount: Decimal) -> Result<()> {
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

    fn withdraw(&mut self, amount: Decimal) -> Result<()> {
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

    fn dispute(&mut self, tx: &Transaction) -> Result<()> {
        if self.dispute_transactions.contains_key(&tx.tx()) {
            bail!("Transaction already disputed");
        }

        let transaction = self
            .effect_transactions
            .get(&tx.tx())
            .ok_or_else(|| anyhow!("Transaction not found"))?
            .clone();

        let amount = transaction.get_amount();

        if !matches!(
            transaction.transaction_type(),
            TransactionType::Deposit { .. }
        ) {
            bail!("Only deposit transactions can be disputed");
        }

        self.effect_transactions.remove(&tx.tx());

        self.available -= amount;
        self.withheld += amount;
        self.dispute_transactions
            .insert(tx.tx(), transaction.clone());
        Ok(())
    }

    fn resolve(&mut self, tx: &Transaction) -> Result<()> {
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

    fn chargeback(&mut self, tx: &Transaction) -> Result<()> {
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
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.00),
            },
        );

        let result = account.process_transaction(deposit_tx);

        assert!(result.is_err());
        assert!(account.available == Decimal::ZERO);
    }

    #[test]
    fn deposit_negative_amount_fails() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(-0.001),
            },
        );

        let result = account.process_transaction(deposit_tx);

        assert!(result.is_err());
        assert!(account.available == dec!(0.0));
    }

    #[test]
    fn deposit_increases_available_and_total() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.01),
            },
        );

        account.process_transaction(deposit_tx).unwrap();

        assert_eq!(account.available, dec!(100.01));
        assert_eq!(account.total, dec!(100.01));
    }

    #[test]
    fn withdraw_locked_account_fails() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.0),
            },
        );
        account.process_transaction(deposit_tx).unwrap();
        account.locked = true;
        let withdraw_tx =
            Transaction::new(1, 2, TransactionType::Withdrawal { amount: dec!(10.0) });

        let result = account.process_transaction(withdraw_tx);

        assert!(result.is_err());
        assert!(account.available == dec!(100.0));
    }

    #[test]
    fn withdraw_negative_amount_fails() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.0),
            },
        );
        account.process_transaction(deposit_tx).unwrap();
        let withdraw_tx = Transaction::new(
            1,
            2,
            TransactionType::Withdrawal {
                amount: dec!(-0.001),
            },
        );

        let result = account.process_transaction(withdraw_tx);

        assert!(result.is_err());
        assert!(account.available == dec!(100.0));
    }

    #[test]
    fn withdraw_decreases_available_and_total() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.01),
            },
        );
        account.process_transaction(deposit_tx).unwrap();
        let withdraw_tx =
            Transaction::new(1, 2, TransactionType::Withdrawal { amount: dec!(50.0) });

        let result = account.process_transaction(withdraw_tx);

        assert_eq!(account.available, dec!(50.01));
        assert_eq!(account.total, dec!(50.01));
        assert!(result.is_ok());
    }

    #[test]
    fn dispute_cannot_dispute_transaction_twice() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.0),
            },
        );

        account.process_transaction(deposit_tx).unwrap();
        let tx = Transaction::new(1, 1, TransactionType::Dispute);
        account.process_transaction(tx.clone()).unwrap();

        let result = account.process_transaction(tx);

        assert!(result.is_err());
    }

    #[test]
    fn dispute_transaction_not_found_fails() {
        let mut account = Account::new(1);
        let tx = Transaction::new(1, 1, TransactionType::Dispute);

        let result = account.process_transaction(tx);

        assert!(result.is_err());
    }

    #[test]
    fn dispute_can_decrease_balance_to_debt() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.0),
            },
        );

        account.process_transaction(deposit_tx).unwrap();

        let withdraw_tx = Transaction::new(
            1,
            2,
            TransactionType::Withdrawal {
                amount: dec!(100.0),
            },
        );
        account.process_transaction(withdraw_tx).unwrap();

        let tx = Transaction::new(1, 1, TransactionType::Dispute);

        let result = account.process_transaction(tx);

        assert!(result.is_ok());
        assert_eq!(account.available, dec!(-100.0));
    }

    #[test]
    fn dispute_only_deposit_transactions() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.0),
            },
        );
        account.process_transaction(deposit_tx).unwrap();

        let withdraw_tx =
            Transaction::new(1, 2, TransactionType::Withdrawal { amount: dec!(50.0) });
        account.process_transaction(withdraw_tx).unwrap();
        let tx = Transaction::new(1, 2, TransactionType::Dispute);

        let result = account.process_transaction(tx);

        assert!(result.is_err());
    }

    #[test]
    fn dispute_lowers_available_and_increases_withheld() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.0),
            },
        );

        account.process_transaction(deposit_tx).unwrap();

        assert_eq!(account.available, dec!(100.0));
        assert_eq!(account.withheld, dec!(0.0));

        let tx = Transaction::new(1, 1, TransactionType::Dispute);

        account.process_transaction(tx).unwrap();

        assert_eq!(account.available, dec!(0.0));
        assert_eq!(account.withheld, dec!(100.0));
    }

    #[test]
    fn resolve_transaction_not_disputed_fails() {
        let mut account = Account::new(1);
        let tx = Transaction::new(1, 1, TransactionType::Resolve);

        let result = account.process_transaction(tx);

        assert!(result.is_err());
    }

    #[test]
    fn resolve_increases_available_and_decreases_withheld() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.0),
            },
        );
        account.process_transaction(deposit_tx).unwrap();

        let dispute_tx = Transaction::new(1, 1, TransactionType::Dispute);
        account.process_transaction(dispute_tx).unwrap();

        assert_eq!(account.available, dec!(0.0));
        assert_eq!(account.withheld, dec!(100.0));

        let resolve_tx = Transaction::new(1, 1, TransactionType::Resolve);
        account.process_transaction(resolve_tx).unwrap();

        assert_eq!(account.available, dec!(100.0));
        assert_eq!(account.withheld, dec!(0.0));
    }

    #[test]
    fn chargeback_transaction_not_disputed_fails() {
        let mut account = Account::new(1);
        let tx = Transaction::new(1, 1, TransactionType::Chargeback);

        let result = account.process_transaction(tx);

        assert!(result.is_err());
    }

    #[test]
    fn chargeback_decreases_total_and_withheld() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.0),
            },
        );
        account.process_transaction(deposit_tx).unwrap();

        let dispute_tx = Transaction::new(1, 1, TransactionType::Dispute);
        account.process_transaction(dispute_tx).unwrap();

        assert_eq!(account.total, dec!(100.0));
        assert_eq!(account.withheld, dec!(100.0));

        let chargeback_tx = Transaction::new(1, 1, TransactionType::Chargeback);
        account.process_transaction(chargeback_tx).unwrap();

        assert_eq!(account.total, dec!(0.0));
        assert_eq!(account.withheld, dec!(0.0));
    }

    #[test]
    fn chargeback_locks_account() {
        let mut account = Account::new(1);
        let deposit_tx = Transaction::new(
            1,
            1,
            TransactionType::Deposit {
                amount: dec!(100.0),
            },
        );
        account.process_transaction(deposit_tx).unwrap();

        let dispute_tx = Transaction::new(1, 1, TransactionType::Dispute);
        account.process_transaction(dispute_tx).unwrap();

        assert!(!account.locked);

        let chargeback_tx = Transaction::new(1, 1, TransactionType::Chargeback);
        account.process_transaction(chargeback_tx).unwrap();

        assert!(account.locked);
    }
}
