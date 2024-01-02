package main

import (
	"errors"
	"fmt"
	"io"
	"os"
	"slices"
)

func main() {
	if err := run(); err != nil {
		panic(err)
	}
}

func run() error {
	file, err := os.Open("nordnet.csv")
	if err != nil {
		return fmt.Errorf("could not open file: %w", err)
	}
	defer file.Close()

	decoder, err := NewNordnetDecoder(file)
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
	slices.Reverse(transactions)

	converter := NewConvert(transactions, 17)
	records := converter.Convert()

	if err := WriteLedger(records); err != nil {
		return fmt.Errorf("could not write ledger: %w", err)
	}

	return nil
}

var ErrEOF = errors.New("EOF")

type Converter struct {
	dineroItems         []DineroRecord
	nordnetTransactions []NordnetTransaction
	readPosition        int
	currentVoucher      int
}

func NewConvert(transactions []NordnetTransaction, nextVoucher int) *Converter {
	return &Converter{
		dineroItems:         []DineroRecord{},
		nordnetTransactions: transactions,
		currentVoucher:      nextVoucher,
		readPosition:        0,
	}
}

func (c *Converter) nextVoucher() int {
	next := c.currentVoucher
	c.currentVoucher++
	return next
}

func (c *Converter) next() (NordnetTransaction, error) {
	next := c.readPosition
	if next >= len(c.nordnetTransactions) {
		return NordnetTransaction{}, ErrEOF
	}

	c.readPosition++
	return c.nordnetTransactions[next], nil
}

func (c *Converter) peak() NordnetTransaction {
	if c.readPosition >= len(c.nordnetTransactions) {
		return NordnetTransaction{}
	}

	return c.nordnetTransactions[c.readPosition]
}

func (c *Converter) Convert() []DineroRecord {
	for {
		transaction, err := c.next()
		if err == ErrEOF {
			break
		}

		switch transaction.TransactionType {
		case "KØBT":
			c.purchase(transaction)
		case "UDB.":
			c.dividend(transaction)
		case "DEPOTRENTE":
			c.interest(transaction)
		}
	}

	return c.dineroItems
}

func (c *Converter) purchase(transaction NordnetTransaction) {
	voucherNumber := c.nextVoucher()
	record := DineroRecord{
		Number:                 voucherNumber,
		Date:                   transaction.Date.Format("02/01/2006"),
		Text:                   fmt.Sprintf("Køb af %s, ISIN: %s", transaction.Company, transaction.ISIN),
		Account:                "55020",
		AccountVatType:         "Ingen moms",
		BalanceAccount:         "51515",
		BalanaceACcountVatType: "Ingen moms",
		Amount:                 DanishAmount(transaction.Total + transaction.TransactionFee),
		ForeignAmount:          DanishAmount(0),
	}
	c.dineroItems = append(c.dineroItems, record)

	if transaction.TransactionFee > 0 {
		fees := DineroRecord{
			Number:                 voucherNumber,
			Date:                   transaction.Date.Format("02/01/2006"),
			Text:                   "Kurtage af køb",
			Account:                "55020",
			AccountVatType:         "Ingen moms",
			BalanceAccount:         "7220",
			BalanaceACcountVatType: "Ingen moms",
			Amount:                 DanishAmount(transaction.TransactionFee * -1),
			ForeignAmount:          DanishAmount(0),
		}
		c.dineroItems = append(c.dineroItems, fees)
	}
}

func (c *Converter) dividend(transaction NordnetTransaction) {
	voucherNumber := c.nextVoucher()
	record := DineroRecord{
		Number:                 voucherNumber,
		Date:                   transaction.Date.Format("02/01/2006"),
		Text:                   fmt.Sprintf("Udbytte - %s, ISIN: %s", transaction.Company, transaction.ISIN),
		Account:                "55020",
		AccountVatType:         "Ingen moms",
		BalanceAccount:         "9020",
		BalanaceACcountVatType: "Ingen moms",
		Amount:                 DanishAmount(transaction.Total + transaction.TransactionFee),
		ForeignAmount:          DanishAmount(0),
	}
	c.dineroItems = append(c.dineroItems, record)

	if c.peak().TransactionType == "UDBYTTESKAT" {
		taxTransaction, _ := c.next()

		tax := DineroRecord{
			Number:                 voucherNumber,
			Date:                   taxTransaction.Date.Format("02/01/2006"),
			Text:                   fmt.Sprintf("Udbytteskat - %s, ISIN: %s", taxTransaction.Company, taxTransaction.ISIN),
			Account:                "55020",
			AccountVatType:         "Ingen moms",
			BalanceAccount:         "9020",
			BalanaceACcountVatType: "Ingen moms",
			Amount:                 DanishAmount(taxTransaction.Total + taxTransaction.TransactionFee),
			ForeignAmount:          DanishAmount(0),
		}
		c.dineroItems = append(c.dineroItems, tax)
	}
}

func (c *Converter) interest(transaction NordnetTransaction) {
	voucherNumber := c.nextVoucher()
	record := DineroRecord{
		Number:                 voucherNumber,
		Date:                   transaction.Date.Format("02/01/2006"),
		Text:                   "Nordnet Renter",
		Account:                "55020",
		AccountVatType:         "Ingen moms",
		BalanceAccount:         "9200",
		BalanaceACcountVatType: "Ingen moms",
		Amount:                 DanishAmount(transaction.Total + transaction.TransactionFee),
		ForeignAmount:          DanishAmount(0),
	}
	c.dineroItems = append(c.dineroItems, record)
}
