package main

import (
	"encoding/csv"
	"fmt"
	"io"
	"os"

	"github.com/jszwec/csvutil"
	"golang.org/x/text/encoding/unicode"
	"golang.org/x/text/transform"
)

type NordnetTransaction struct {
	Date            string `csv:"Bogføringsdag"`
	Company         string `csv:"Værdipapirer"`
	ISIN            string `csv:"ISIN"`
	TransactionType string `csv:"Transaktionstype"`
	TransactionText string `csv:"Transaktionstekst"`
	Price           string `csv:"Kurs"`
	TransactionFee  string `csv:"Samlede afgifter"`
	Total           string `csv:"Beløb"`
	Count           string `csv:"Antal"`
}

type DineroRecord struct {
	Number                 int    `csv:"Bilag nr."`
	Date                   string `csv:"Dato"`
	Text                   string `csv:"Tekst"`
	Account                string `csv:"Konto"`
	AccountVatType         string `csv:"Konto momstype"`
	Amount                 string `csv:"Beløb"`
	ForeignAmount          string `csv:"Beløb udenlandsk valuta"`
	BalanceAccount         string `csv:"Modkonto"`
	BalanaceACcountVatType string `csv:"Modkonto momstype"`
}

func main() {
	if err := run(); err != nil {
		panic(err)
	}
}

func run() error {
	nt, err := os.Open("nordnet.csv")
	if err != nil {
		return fmt.Errorf("could not open file: %w", err)
	}
	defer nt.Close()

	decoder, err := NewNordnetDecoder(nt)
	if err != nil {
		return fmt.Errorf("could not create decoder: %w", err)
	}

	var transactions []NordnetTransaction
	for {
		var t NordnetTransaction
		if err := decoder.Decode(&t); err == io.EOF {
			break
		} else if err != nil {
			return fmt.Errorf("could not decode transaction: %w", err)
		}
		transactions = append(transactions, t)
	}

	for _, t := range transactions {
		fmt.Println(t)
	}

	return nil
}

type Converter struct {
	DineroItems         string
	NordnetTransactions []NordnetTransaction
	NextVoucher         int
}

func NewUTF16LEReader(r io.Reader) io.Reader {
	winutf := unicode.UTF16(unicode.LittleEndian, unicode.IgnoreBOM)
	decoder := winutf.NewDecoder()

	unicodeReader := transform.NewReader(r, decoder)

	return unicodeReader
}

func NewNordnetDecoder(r io.Reader) (*csvutil.Decoder, error) {
	utf16_reader := NewUTF16LEReader(r)
	csvReader := csv.NewReader(utf16_reader)
	csvReader.Comma = '\t'

	dec, err := csvutil.NewDecoder(csvReader)
	if err != nil {
		return nil, fmt.Errorf("could not create CSV decoder: %w", err)
	}

	return dec, nil
}
