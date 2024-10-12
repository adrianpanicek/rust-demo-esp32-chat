use serde::Serialize;

#[derive(Serialize)]
pub struct Message{
    pub id: u32,
    pub author: String,
    pub message: String
}