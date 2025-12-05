use anyhow::{anyhow, bail};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::models::transaction::{Transaction, TransactionType};

#[derive(Clone, Debug, Deserialize)]
pub struct TransactionRecord {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

impl TryFrom<TransactionRecord> for Transaction {
    type Error = anyhow::Error;

    fn try_from(record: TransactionRecord) -> Result<Self, Self::Error> {
        match record.transaction_type.as_str() {
            "deposit" => Ok(Transaction::new(
                record.client,
                record.tx,
                TransactionType::Deposit {
                    amount: Decimal::from_f64_retain(
                        record
                            .amount
                            .ok_or_else(|| anyhow!("Amount is required for deposit"))?,
                    )
                    .ok_or_else(|| anyhow!("Invalid amount for deposit"))?,
                },
            )),
            "withdrawal" => Ok(Transaction::new(
                record.client,
                record.tx,
                TransactionType::Withdrawal {
                    amount: Decimal::from_f64_retain(
                        record
                            .amount
                            .ok_or_else(|| anyhow!("Amount is required for withdrawal"))?,
                    )
                    .ok_or_else(|| anyhow!("Invalid amount for withdrawal"))?,
                },
            )),
            "dispute" => Ok(Transaction::new(
                record.client,
                record.tx,
                TransactionType::Dispute,
            )),
            "resolve" => Ok(Transaction::new(
                record.client,
                record.tx,
                TransactionType::Resolve,
            )),
            "chargeback" => Ok(Transaction::new(
                record.client,
                record.tx,
                TransactionType::Chargeback,
            )),
            other => bail!("Unknown transaction type: {}", other),
        }
    }
}
