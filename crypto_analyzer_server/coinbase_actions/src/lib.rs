use serde::Serialize;

#[derive(Serialize)]
pub struct ServerResponse<C: serde::Serialize> {
    execution_time_milis: u32,
    response: C,
}
