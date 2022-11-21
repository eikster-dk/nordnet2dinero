use std::error::Error;
use std::fs::File;
use std::io;
use std::process;

use serde::Deserialize;

use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;

#[derive(Debug, Deserialize)]
struct nordnet_record {
    #[serde(rename = "Bogføringsdag")]
    date: String,

    #[serde(rename = "Værdipapirer")]
    company: String,

    #[serde(rename = "Transaktionstype")]
    transaction_type: nordnet_transaction_type,

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
enum nordnet_transaction_type {
    #[serde(rename = "KØBT")]
    purchase,

    #[serde(rename = "UDB.")]
    dividend,

    #[serde(rename = "UDBYTTESKAT")]
    dividend_tax,

    #[serde(rename = "DEPOTRENTE")]
    interest,

    #[serde(rename = "INDBETALING")]
    payment,
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
        let record: nordnet_record = result?;
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
