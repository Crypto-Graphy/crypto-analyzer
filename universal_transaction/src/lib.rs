extern crate coinbase_transactions;
extern crate kraken_ledgers;
extern crate rust_decimal;

use coinbase_transactions::transaction_parser::CoinbaseTransactionRecord;
use kraken_ledgers::ledger_parser::KrakenLedgerRecord;

use self::rust_decimal::Decimal;

#[derive(Debug, PartialEq, Eq)]
pub enum TransactionSource {
    Coinbase,
    Kraken,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PriceTransaction {
    pub source: TransactionSource,
    pub currency: String,
    pub quantity_of_currency: Decimal,
    pub date_of_transaction: String,
    pub currency_price_at_date: Option<Decimal>,
    pub comparison_currency: String,
}

impl From<CoinbaseTransactionRecord> for PriceTransaction {
    /// Converts a CoinbaseTransactionRecord into a PriceTransaction.
    ///
    /// ```
    /// # extern crate rust_decimal;
    /// # extern crate coinbase_transactions;
    ///
    /// # use coinbase_transactions::transaction_parser::CoinbaseTransactionRecord;
    /// # use universal_transaction::PriceTransaction;
    /// # use universal_transaction::TransactionSource;
    /// # use rust_decimal::Decimal;
    /// #
    /// let c_transaction = CoinbaseTransactionRecord {
    /// time_of_transaction: "2021-04-01T21:38:01Z".to_string(),
    /// transaction_type: "Buy".to_string(),
    /// asset: "BTC".to_string(),
    /// // ... the other properties
    /// # quantity_transacted: Decimal::new(00016458, 7),
    /// # spot_price_currency: "USD".to_string(),
    /// # spot_price_at_transaction: Some(Decimal::new(5894398, 2)),
    /// # subtotal: Some(Decimal::new(9701, 2)),
    /// # total: Some(Decimal::new(100, 0)),
    /// # fees: Some(Decimal::new(299, 2)),
    /// # notes: "Bought 0.0016458 BTC for $100.00 USD".to_string(),
    /// };
    /// let expected = PriceTransaction {
    ///     source: TransactionSource::Coinbase,
    ///     currency: "BTC".to_string(),
    ///     quantity_of_currency: Decimal::new(00016458, 7),
    ///     date_of_transaction: "2021-04-01T21:38:01Z".to_string(),
    ///     currency_price_at_date: None,
    ///     comparison_currency: "usd".to_string()
    /// };
    ///
    /// assert_eq!(PriceTransaction::from(c_transaction), expected);
    /// ```
    fn from(transaction: CoinbaseTransactionRecord) -> Self {
        PriceTransaction {
            source: TransactionSource::Coinbase,
            currency: transaction.asset.to_string(),
            quantity_of_currency: transaction.quantity_transacted,
            date_of_transaction: transaction.time_of_transaction,
            currency_price_at_date: None,
            comparison_currency: String::from("usd"),
        }
    }
}

impl From<KrakenLedgerRecord> for PriceTransaction {
    /// Converts a KrakenLedgerRecord into a PriceTransaction.
    ///
    /// ```
    /// # extern crate rust_decimal;
    /// # extern crate kraken_ledgers;
    /// # extern crate universal_transaction;
    ///
    /// # use rust_decimal::Decimal;
    /// # use kraken_ledgers::ledger_parser::KrakenLedgerRecord;
    /// # use universal_transaction::PriceTransaction;
    /// # use universal_transaction::TransactionSource;
    /// let k_transaction = KrakenLedgerRecord {
    ///     txid: Some("L7RLII-4423D-JTUU7J".to_string()),
    ///     refid: "L7RLII-4423D-JTUU9E".to_string(),
    ///     time: "2021-09-29 15:18:30".to_string(),
    ///     record_type: "deposit".to_string(),
    ///     // ... other properties
    ///     # subtype: None,
    ///     # a_class: "currency".to_string(),
    ///     # asset: "ADA".to_string(),
    ///     # amount: Decimal::new(2323, 2),
    ///     # fee: Decimal::new(0, 0),
    ///     # balance: Some(Decimal::new(2323, 2)),
    /// };
    ///
    /// let expected = PriceTransaction {
    ///     source: TransactionSource::Kraken,
    ///     currency: String::from("ADA"),
    ///     quantity_of_currency: Decimal::new(2323, 2),
    ///     date_of_transaction: "2021-09-29 15:18:30".to_string(),
    ///     currency_price_at_date: None,
    ///     comparison_currency: String::from("usd"),
    /// };
    ///
    /// assert_eq!(PriceTransaction::from(k_transaction), expected);
    /// ```
    fn from(record: KrakenLedgerRecord) -> Self {
        PriceTransaction {
            source: TransactionSource::Kraken,
            currency: record.asset,
            quantity_of_currency: record.amount,
            date_of_transaction: record.time,
            currency_price_at_date: None,
            comparison_currency: String::from("usd"),
        }
    }
}
