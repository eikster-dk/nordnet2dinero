use std::error::Error;
use std::fs::File;
use std::io;
use std::iter::Peekable;
use std::process;

use chrono::Datelike;
use chrono::NaiveDate;
use csv::WriterBuilder;
use serde::Deserialize;

use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Serialize;

#[derive(Debug, Deserialize, Clone)]
struct NordnetRecord {
    #[serde(rename = "Bogføringsdag")]
    date: NaiveDate,

    #[serde(rename = "Værdipapirer")]
    company: String,

    #[serde(rename = "ISIN")]
    isin: String,

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

impl NordnetRecord {
    fn get_fees(&self) -> f64 {
        self.transaction_fees
            .trim()
            .replace(".", "")
            .replace(",", ".")
            .parse()
            .unwrap()
    }

    fn get_total(&self) -> f64 {
        let fees = self.get_fees();
        let total: f64 = self
            .total
            .trim()
            .replace(".", "")
            .replace(",", ".")
            .parse()
            .unwrap();

        // total is negative, fees are positive
        total + fees
    }
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Serialize)]
struct DineroRecord {
    #[serde(rename = "Bilag nr.")]
    number: isize,

    #[serde(rename = "Dato")]
    date: String,

    #[serde(rename = "Tekst")]
    text: String,

    #[serde(rename = "Konto")]
    account: String,

    #[serde(rename = "Konto momstype")]
    account_vat_type: String,

    #[serde(rename = "Beløb")]
    amount: String,

    #[serde(rename = "Beløb udenlandsk valuta")]
    foreign_amount: String,

    #[serde(rename = "Modkonto")]
    balance_account: String,

    #[serde(rename = "Modkonto momstype")]
    balance_account_vat_type: String,
}

fn read_nordnet_csv() -> Result<Vec<NordnetRecord>, Box<dyn Error>> {
    let file = File::open("test.csv")?;
    let transcoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(UTF_16LE))
        .build(file);

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(transcoded);

    let mut items: Vec<NordnetRecord> = Vec::new();

    for result in reader.deserialize() {
        let record: NordnetRecord = result?;
        items.push(record);
    }

    items.reverse();
    Ok(items)
}

fn write_dinero_records(records: Vec<DineroRecord>) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(io::stdout());

    for r in records {
        wtr.serialize(r)?
    }

    Ok(())
}

struct converter {
    nordnet_iter: Peekable<std::vec::IntoIter<NordnetRecord>>,
    DineroItems: Vec<DineroRecord>,
    VoucherNumber: isize,
}

impl converter {
    fn new(items: Vec<NordnetRecord>, next: isize) -> Self {
        converter {
            nordnet_iter: items.into_iter().peekable(),
            DineroItems: vec![],
            VoucherNumber: next,
        }
    }

    fn next(&mut self) -> isize {
        let next = self.VoucherNumber;
        self.VoucherNumber += 1;

        next
    }

    fn convert(&mut self) -> Vec<DineroRecord> {
        let mut records = vec![];
        while let Some(record) = self.nordnet_iter.next() {
            let mut items = self.convert_nordnet_record(record);
            records.append(&mut items);
        }

        records
    }

    fn convert_nordnet_record(&mut self, r: NordnetRecord) -> Vec<DineroRecord> {
        match r.transaction_type {
            NordnetTransactionType::Purchase => self.convert_purchase(r),
            NordnetTransactionType::Dividend => self.convert_dividend(r),
            NordnetTransactionType::Interest => self.convert_interest(r),
            _ => vec![],
        }
    }

    fn convert_purchase(&mut self, r: NordnetRecord) -> Vec<DineroRecord> {
        let number = self.next();
        let mut records = vec![];

        let purchase = DineroRecord {
            date: format!("{}/{}/{}", r.date.day(), r.date.month(), r.date.year()),
            text: format!("Køb af {}, ISIN: {}", r.company, r.isin),
            number: number,
            account: 55020.to_string(),
            account_vat_type: "Ingen moms".to_string(),
            amount: r.get_total().to_string().replace(".", ","),
            foreign_amount: "0,0".to_string(),
            balance_account: 51515.to_string(),
            balance_account_vat_type: "Ingen moms".to_string(),
        };
        records.push(purchase);

        if r.get_fees() > 0.0 {
            let fees = DineroRecord {
                date: format!("{}/{}/{}", r.date.day(), r.date.month(), r.date.year()),
                text: format!("Kurtage af køb"),
                number: number,
                account: 55020.to_string(),
                account_vat_type: "Ingen moms".to_string(),
                amount: (r.get_fees() * -1.0).to_string().replace(".", ","),
                foreign_amount: "0,0".to_string(),
                balance_account: 7220.to_string(),
                balance_account_vat_type: "Ingen moms".to_string(),
            };

            records.push(fees);
        }

        records
    }

    fn convert_dividend(&mut self, r: NordnetRecord) -> Vec<DineroRecord> {
        let number = self.next();

        let mut records = vec![];

        let dividend = DineroRecord {
            date: format!("{}/{}/{}", r.date.day(), r.date.month(), r.date.year()),
            text: format!("Udbytte - {}, ISIN: {}", r.company, r.isin),
            number: number,
            account: 55020.to_string(),
            account_vat_type: "Ingen moms".to_string(),
            amount: r.get_total().to_string().replace(".", ","),
            foreign_amount: "0,0".to_string(),
            balance_account: 9020.to_string(),
            balance_account_vat_type: "Ingen moms".to_string(),
        };
        records.push(dividend);

        if let Some(record) = self.nordnet_iter.peek() {
            match record.transaction_type {
                NordnetTransactionType::DividendTax => {
                    let tax = DineroRecord {
                        date: format!("{}/{}/{}", r.date.day(), r.date.month(), r.date.year()),
                        text: format!("Udbytteskat - {}, ISIN: {}", r.company, r.isin),
                        number: number,
                        account: 55020.to_string(),
                        account_vat_type: "Ingen moms".to_string(),
                        amount: record.get_total().to_string().replace(".", ","),
                        foreign_amount: "0,0".to_string(),
                        balance_account: 54055.to_string(),
                        balance_account_vat_type: "Ingen moms".to_string(),
                    };
                    records.push(tax);
                    self.nordnet_iter.next();
                }
                _ => (),
            }
        }

        records
    }

    fn convert_interest(&mut self, r: NordnetRecord) -> Vec<DineroRecord> {
        let number = self.next();
        let interest = DineroRecord {
            date: format!("{}/{}/{}", r.date.day(), r.date.month(), r.date.year()),
            text: format!("Renter"),
            number: number,
            account: 55020.to_string(),
            account_vat_type: "Ingen moms".to_string(),
            amount: r.get_total().to_string().replace(".", ","),
            foreign_amount: "0,0".to_string(),
            balance_account: 9200.to_string(),
            balance_account_vat_type: "Ingen moms".to_string(),
        };

        vec![interest]
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let nordnet_records = read_nordnet_csv()?;
    let mut converter = converter::new(nordnet_records, 67);
    let dinero_records = converter.convert();

    write_dinero_records(dinero_records)?;

    Ok(())
}
