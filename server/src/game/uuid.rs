use super::super::auth::SESSION_COOKIE_NAME;
use super::Error;
use serde::Serialize;
use std::str::FromStr;
use std::string::ToString;
use uuid::Uuid;

macro_rules! uuid {
    ($struct_name:ident) => {
        #[derive(Clone, PartialEq, Eq, Hash, Serialize, Debug, Default)]
        pub struct $struct_name(Uuid);

        impl $struct_name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl ToString for $struct_name {
            fn to_string(&self) -> String {
                let mut buf = [b'!'; 36];
                self.0.to_simple().encode_lower(&mut buf).to_string()
            }
        }

        impl FromStr for $struct_name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl<'a> rocket::request::FromParam<'a> for $struct_name {
            type Error = uuid::Error;
            fn from_param(param: &'a str) -> Result<Self, Self::Error> {
                Ok(Self(Uuid::parse_str(param)?))
            }
        }

        impl<'a> rocket::form::FromFormField<'a> for $struct_name {
            fn from_value(field: rocket::form::ValueField<'a>) -> rocket::form::Result<'a, Self> {
                match Uuid::parse_str(field.value) {
                    Ok(uuid) => Ok(Self(uuid)),
                    Err(_) => Err(rocket::form::Error::validation("Not a valid UUID").into()),
                }
            }
        }
    };
}

uuid!(PlayerUUID);
uuid!(GameUUID);

impl PlayerUUID {
    pub fn from_cookie_jar(cookie_jar: &rocket::http::CookieJar) -> Result<Self, Error> {
        match cookie_jar.get(SESSION_COOKIE_NAME) {
            Some(cookie) => match Self::from_str(cookie.value()) {
                Ok(player_uuid) => Ok(player_uuid),
                Err(_) => Err(Error::new("User is not signed in")),
            },
            None => Err(Error::new("User is not signed in")),
        }
    }

    pub fn to_cookie_jar(&self, cookie_jar: &rocket::http::CookieJar) {
        cookie_jar.remove(rocket::http::Cookie::named(SESSION_COOKIE_NAME));
        cookie_jar.add(rocket::http::Cookie::new(
            SESSION_COOKIE_NAME,
            self.to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_convert_to_and_from_string() {
        uuid!(TestUUID);

        let test_uuid = TestUUID::new();
        assert_eq!(
            test_uuid,
            TestUUID::from_str(&test_uuid.to_string()).unwrap()
        );

        // Stringified version is a 32-character hex string.
        assert!(TestUUID::from_str("1bc68e20bad1456dab8039137094ca6d").is_ok());
    }

    #[test]
    fn generates_unique_ids() {
        uuid!(TestUUID);

        let test_uuid_1 = TestUUID::new();
        assert!(test_uuid_1 == test_uuid_1.clone()); // Sanity check.
        let test_uuid_2 = TestUUID::new();
        assert!(test_uuid_1 != test_uuid_2);
    }
}
