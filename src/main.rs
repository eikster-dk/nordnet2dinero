use std::error::Error;
use std::fs::File;
use std::io;
use std::process;

use serde::Deserialize;

use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;

#[derive(Debug, Deserialize)]
struct NordnetRecord {
    #[serde(rename = "Bogføringsdag")]
    date: String,

    #[serde(rename = "Værdipapirer")]
    company: String,

    #[serde(rename = "Transaktionstype")]
    transaction_type: NordnetTransactionType,

    #[serde(rename = "Transaktionstekst")]
    transaction_text: String,

    #[serde(rename = "Antal")]
    count: isize,

    #[serde(rename = "Kurs")]
    price: String,

    #[serde(rename = "Samlede afgifter")]
    transaction_fees: String,

    #[serde(rename = "Beløb")]
    total: String,
}

#[derive(Debug, Deserialize)]
enum NordnetTransactionType {
    #[serde(rename = "KØBT")]
    Purchase,

    #[serde(rename = "UDB.")]
    Dividend,

    #[serde(rename = "UDBYTTESKAT")]
    DividendTax,

    #[serde(rename = "DEPOTRENTE")]
    Interest,

    #[serde(rename = "INDBETALING")]
    Payment,
}

fn read_nordnet_csv() -> Result<(), Box<dyn Error>> {
    let file = File::open("test.csv")?;
    let transcoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(UTF_16LE))
        .build(file);

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(transcoded);

    for result in reader.deserialize() {
        let record: NordnetRecord = result?;
        println!("{:?}", record);
    }
    Ok(())
}

fn main() {
    if let Err(err) = read_nordnet_csv() {
        println!("error running nordnet_csv: {}", err);
        process::exit(1);
    }
}
