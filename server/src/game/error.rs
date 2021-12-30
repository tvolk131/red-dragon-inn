pub struct Error(String);

impl Error {
    pub fn new(message: impl ToString) -> Self {
        Self(message.to_string())
    }
}
