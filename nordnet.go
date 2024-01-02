package main

import (
	"encoding/csv"
	"fmt"
	"io"
	"strconv"
	"strings"
	"time"

	"github.com/jszwec/csvutil"
	"golang.org/x/text/encoding/unicode"
	"golang.org/x/text/transform"
)

type NordnetTransaction struct {
	Date            time.Time     `csv:"Bogføringsdag"`
	Company         string        `csv:"Værdipapirer"`
	ISIN            string        `csv:"ISIN"`
	TransactionType string        `csv:"Transaktionstype"`
	TransactionText string        `csv:"Transaktionstekst"`
	Count           string        `csv:"Antal"`
	Price           NordnetAmount `csv:"Kurs"`
	TransactionFee  NordnetAmount `csv:"Samlede afgifter"`
	Total           NordnetAmount `csv:"Beløb"`
}

type NordnetAmount float64

func (f *NordnetAmount) UnmarshalText(data []byte) error {
	s := string(data)
	s = strings.ReplaceAll(s, ".", "")
	s = strings.ReplaceAll(s, ",", ".")

	i, err := strconv.ParseFloat(s, 64)
	if err != nil {
		return err
	}

	*f = NordnetAmount(i)
	return nil
}

func NordnetTime(data []byte, t *time.Time) error {
	format := "2006-01-02"
	tt, err := time.Parse(format, string(data))
	if err != nil {
		return err
	}
	*t = tt
	return nil
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
	dec.Register(NordnetTime)

	return dec, nil
}
