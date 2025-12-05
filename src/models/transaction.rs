use rust_decimal::Decimal;

#[derive(Clone, Debug)]
pub struct Transaction {
    client: u16,
    tx: u32,
    transaction_type: TransactionType,
}

#[derive(Clone, Debug)]
pub enum TransactionType {
    Deposit { amount: Decimal },
    Withdrawal { amount: Decimal },
    Dispute,
    Resolve,
    Chargeback,
}

impl Transaction {
    pub fn new(client: u16, tx: u32, transaction_type: TransactionType) -> Self {
        Transaction {
            client,
            tx,
            transaction_type,
        }
    }

    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn tx(&self) -> u32 {
        self.tx
    }

    pub fn transaction_type(&self) -> &TransactionType {
        &self.transaction_type
    }

    pub fn get_amount(&self) -> Decimal {
        match &self.transaction_type {
            TransactionType::Deposit { amount } => *amount,
            TransactionType::Withdrawal { amount } => *amount,
            _ => Decimal::ZERO,
        }
    }
}
