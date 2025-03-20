use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseBody<T> {
    pub message: String,
    pub data: Option<T>,
}

impl<T> ResponseBody<T> {
    pub fn new(message: &str, data: Option<T>) -> Self {
        Self {
            message: message.to_string(),
            data,
        }
    }
}
