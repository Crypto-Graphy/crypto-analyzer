use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ServerResponse<T>
where
    T: serde::Serialize,
{
    pub transaction_id: Uuid,
    pub success: bool,
    pub response: Option<T>,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}

impl<T> ServerResponse<T>
where
    T: serde::Serialize,
{
    pub fn new(
        transaction_id: Option<Uuid>,
        success: bool,
        response: Option<T>,
        messages: Option<Vec<String>>,
        errors: Option<Vec<String>>,
    ) -> Self {
        Self {
            transaction_id: transaction_id.unwrap_or(Uuid::new_v4()),
            success,
            response,
            messages: messages.unwrap_or(Default::default()),
            errors: errors.unwrap_or(Default::default()),
        }
    }
}

impl<T> Default for ServerResponse<T>
where
    T: serde::Serialize,
{
    fn default() -> Self {
        Self {
            transaction_id: Uuid::new_v4(),
            success: false,
            response: None,
            messages: Vec::new(),
            errors: Vec::new(),
        }
    }
}
