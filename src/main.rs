use clap::Parser;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fs::File, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    file: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawl,
    Dispute,
    Resolve,
    ChargeBack,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct Client {
    #[serde(rename(serialize = "client"))]
    id: u16,

    #[serde(rename(serialize = "available"))]
    available_amount: Decimal,

    #[serde(rename(serialize = "held"))]
    held_amount: Decimal,

    #[serde(rename(serialize = "total"))]
    total_amount: Decimal,

    locked: bool,
}

impl Client {
    fn new(id: u16) -> Self {
        Self {
            id,
            available_amount: dec!(0),
            held_amount: dec!(0),
            total_amount: dec!(0),
            locked: false,
        }
    }

    // increases available and total funds by amount
    fn deposit(&mut self, amount: Decimal) {
        self.available_amount += amount;
        self.total_amount += amount;
    }

    // decreases available and total funds by amount
    fn withdrawl(&mut self, amount: Decimal) {
        self.available_amount -= amount;
        self.total_amount -= amount;
    }

    // available funds should decrease by amount,
    //    held should increase by amount.
    // total should remain the same
    fn hold(&mut self, amount: Decimal) {
        self.available_amount -= amount;
        self.held_amount += amount;
    }

    // held funds should decrease by the amount
    // available funds should increase by the maount
    // total should remain the same
    fn release(&mut self, amount: Decimal) {
        self.held_amount -= amount;
        self.available_amount += amount;
    }

    fn freeze(&mut self) {
        self.locked = true;
    }
}

// should use the referenced transaction
fn dispute(client: &mut Client, transaction: &Transaction) {
    client.hold(transaction.amount.unwrap());
}

// should use the referenced transaction
fn resolve(client: &mut Client, transaction: &Transaction) {
    client.release(transaction.amount.unwrap());
}

// should use the referenced transaction
fn chargeback(client: &mut Client, transaction: &Transaction) {
    client.withdrawl(transaction.amount.unwrap());
    client.freeze()
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    #[serde(rename(deserialize = "type"))]
    transaction_type: TransactionType,
    client: u16,
    tx: u32,

    #[serde(with = "rust_decimal::serde::arbitrary_precision_option")]
    amount: Option<Decimal>,
}

fn handle_transaction(transaction: Transaction, client_list: &mut HashMap<u16, Client>) {
    if !client_list.contains_key(&transaction.client) {
        client_list.insert(transaction.client, Client::new(transaction.client));
    };
    let client = client_list.get_mut(&transaction.client).unwrap();

    match transaction.transaction_type {
        TransactionType::Deposit => client.deposit(transaction.amount.unwrap()),
        TransactionType::Withdrawl => client.withdrawl(transaction.amount.unwrap()),
        TransactionType::Dispute => dispute(client, &transaction),
        TransactionType::Resolve => resolve(client, &transaction),
        TransactionType::ChargeBack => chargeback(client, &transaction),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let file = File::open(args.file)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut client_list: HashMap<u16, Client> = HashMap::new();
    for result in rdr.deserialize() {
        let transaction: Transaction = result?;
        handle_transaction(transaction, &mut client_list);
    }
    Ok(())
}
