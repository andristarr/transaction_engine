use anyhow::Result;
use csv::{ReaderBuilder, Trim, Writer};

use crate::{engine::Engine, models::transaction_record::TransactionRecord};

pub struct FileRunner;

impl FileRunner {
    pub fn new() -> Self {
        FileRunner
    }

    pub fn run(&self, input_file: &str, engine: &mut Engine) -> Result<()> {
        let mut csv_reader = ReaderBuilder::new().trim(Trim::All).from_path(input_file)?;

        for result in csv_reader.deserialize::<TransactionRecord>() {
            let record = match result {
                Ok(r) => r,
                Err(_) => {
                    continue;
                }
            };
            let transaction = match record.try_into() {
                Ok(t) => t,
                Err(_) => {
                    continue;
                }
            };
            if let Err(_) = engine.process_transaction(transaction) {
                continue;
            }
        }

        self.print_accounts(engine)?;

        Ok(())
    }

    fn print_accounts(&self, engine: &Engine) -> Result<()> {
        let accounts = engine.get_accounts();
        let mut sorted_accounts: Vec<_> = accounts.values().collect();
        sorted_accounts.sort_by_key(|a| a.client);

        let mut wtr = Writer::from_writer(std::io::stdout());

        wtr.write_record(&["client", "available", "held", "total", "locked"])?;

        for account in sorted_accounts {
            wtr.write_record(&[
                account.client.to_string(),
                format!("{:.4}", account.available),
                format!("{:.4}", account.withheld),
                format!("{:.4}", account.total),
                account.locked.to_string(),
            ])?;
        }

        wtr.flush()?;

        Ok(())
    }
}
