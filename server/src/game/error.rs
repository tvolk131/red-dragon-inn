#[derive(Debug, PartialEq)]
pub struct Error(String);

impl Error {
    pub fn new(message: impl ToString) -> Self {
        Self(message.to_string())
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for Error {
    fn respond_to(
        self,
        _request: &'r rocket::request::Request,
    ) -> Result<rocket::response::Response<'static>, rocket::http::Status> {
        rocket::Response::build()
            .status(rocket::http::Status::BadRequest)
            .header(rocket::http::ContentType::Text)
            .sized_body(self.0.len(), std::io::Cursor::new(self.0))
            .ok()
    }
}
