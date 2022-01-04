use super::super::auth::SESSION_COOKIE_NAME;
use super::Error;
use serde::Serialize;

macro_rules! uuid {
    ($struct_name:ident) => {
        #[derive(Clone, PartialEq, Eq, Hash, Serialize)]
        pub struct $struct_name(String);

        impl $struct_name {
            pub fn new() -> Self {
                // TODO - Should generate actual unique id rather than an empty string.
                Self(String::from(""))
            }
        }

        impl std::string::ToString for $struct_name {
            fn to_string(&self) -> String {
                self.0.clone()
            }
        }

        impl<'a> rocket::request::FromParam<'a> for $struct_name {
            type Error = String;
            fn from_param(param: &'a str) -> Result<Self, String> {
                Ok(Self(String::from(param)))
            }
        }

        impl<'a> rocket::form::FromFormField<'a> for $struct_name {
            fn from_value(field: rocket::form::ValueField<'a>) -> rocket::form::Result<'a, Self> {
                Ok(Self(String::from(field.value)))
            }
        }
    }
}

uuid!(PlayerUUID);
uuid!(GameUUID);

impl PlayerUUID {
    pub fn from_cookie_jar(cookie_jar: &rocket::http::CookieJar) -> Result<Self, Error> {
        match cookie_jar.get(SESSION_COOKIE_NAME) {
            Some(cookie) => Ok(Self(String::from(cookie.value()))),
            None => Err(Error::new("User is not signed in")),
        }
    }

    pub fn to_cookie_jar(&self, cookie_jar: &rocket::http::CookieJar) {
        cookie_jar.add(rocket::http::Cookie::new(
            SESSION_COOKIE_NAME,
            self.to_string(),
        ))
    }
}
