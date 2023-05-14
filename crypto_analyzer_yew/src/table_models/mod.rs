use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{de, Deserialize};
use uuid::Uuid;
use yew::{AttrValue, Properties};

#[derive(Properties, PartialEq)]
pub struct KrakenTableTransaction {
    pub id: i32,
    pub txid: Option<AttrValue>,
    pub refid: AttrValue,
    pub transaction_time: DateTime<Utc>,
    pub record_type: AttrValue,
    pub subtype: Option<AttrValue>,
    pub a_class: AttrValue,
    pub asset: AttrValue,
    pub amount: Decimal,
    pub fee: Decimal,
    pub balance: Option<Decimal>,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct KrakenTransaction {
    pub id: i32,
    pub txid: Option<String>,
    pub refid: String,
    pub transaction_time: DateTime<Utc>,
    pub record_type: String,
    pub subtype: Option<String>,
    pub a_class: String,
    pub asset: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub balance: Option<Decimal>,
}

impl From<KrakenTransaction> for KrakenTableTransaction {
    fn from(value: KrakenTransaction) -> Self {
        Self {
            id: value.id,
            txid: value.txid.map(AttrValue::from),
            refid: AttrValue::from(value.refid),
            transaction_time: value.transaction_time,
            record_type: AttrValue::from(value.record_type),
            subtype: value.subtype.map(AttrValue::from),
            a_class: AttrValue::from(value.a_class),
            asset: AttrValue::from(value.asset),
            amount: value.amount,
            fee: value.fee,
            balance: value.balance,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ServerResponse<T>
where
    Box<T>: de::DeserializeOwned,
{
    pub transaction_id: Uuid,
    pub success: bool,
    pub response: Option<T>,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}

// impl<T> ServerResponse<'a, T>
// where
//     T: Deserialize,
// {
//     pub fn new(
//         transaction_id: Option<Uuid>,
//         success: bool,
//         response: Option<T>,
//         messages: Option<Vec<String>>,
//         errors: Option<Vec<String>>,
//     ) -> Self {
//         Self {
//             transaction_id: transaction_id.unwrap_or(Uuid::new_v4()),
//             success,
//             response,
//             messages: messages.unwrap_or(Default::default()),
//             errors: errors.unwrap_or(Default::default()),
//         }
//     }
// }
