package main

import (
	"encoding/csv"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/jszwec/csvutil"
)

type DineroRecord struct {
	Number                 int          `csv:"Bilag nr."`
	Date                   string       `csv:"Dato"`
	Text                   string       `csv:"Tekst"`
	Account                string       `csv:"Konto"`
	AccountVatType         string       `csv:"Konto momstype"`
	Amount                 DanishAmount `csv:"Beløb"`
	ForeignAmount          DanishAmount `csv:"Beløb udenlandsk valuta"`
	BalanceAccount         string       `csv:"Modkonto"`
	BalanaceACcountVatType string       `csv:"Modkonto momstype"`
}

type DanishAmount float64

func (f DanishAmount) MarshalCSV() ([]byte, error) {
	s := strconv.FormatFloat(float64(f), 'f', 2, 64)
	s = strings.ReplaceAll(s, ".", ",")

	return []byte(s), nil
}

func WriteLedger(records []DineroRecord) error {
	f, err := os.Create("output.csv")
	if err != nil {
		return fmt.Errorf("could not create output file: %w", err)
	}

	w := csv.NewWriter(f)
	w.Comma = ';'

	enc := csvutil.NewEncoder(w)
	for _, record := range records {
		if err := enc.Encode(record); err != nil {
			return err
		}
	}
	w.Flush()
	if err := w.Error(); err != nil {
		return err
	}

	return nil
}
