use ammonia::{Builder, UrlRelative};
use serde::{self, Serialize, Deserialize,
    Serializer, Deserializer, de::Visitor};
use std::{fmt::{self, Display},
    borrow::{Borrow, Cow}, io::Write,
    iter, ops::Deref};
use diesel::{self, deserialize::Queryable,
    types::ToSql,
    sql_types::Text,
    serialize::{self, Output}};

lazy_static! {
    static ref CLEAN: Builder<'static> = {
        let mut b = Builder::new();
        b.add_generic_attributes(iter::once("id"))
            .add_tags(iter::once("iframe"))
            .id_prefix(Some("postcontent-"))
            .url_relative(UrlRelative::Custom(Box::new(url_add_prefix)))
            .add_tag_attributes("iframe",
                                ["width", "height", "src", "frameborder"]
                                    .iter()
                                    .map(|&v| v));
        b
    };
}

fn url_add_prefix(url: &str) -> Option<Cow<str>> {
    if url.starts_with('#') && ! url.starts_with("#postcontent-") {//if start with an #
        let mut new_url = "#postcontent-".to_owned();//change to valid id
        new_url.push_str(&url[1..]);
        Some(Cow::Owned(new_url))
    } else {
        Some(Cow::Borrowed(url))
    }
}

#[derive(Debug, Clone, AsExpression, FromSqlRow, Default)]
#[sql_type = "Text"]
pub struct SafeString{
    value: String,
}

impl SafeString{
pub fn new(value: &str) -> Self {
    SafeString{
            value: CLEAN.clean(&value).to_string(),
        }
    }
    pub fn set(&mut self, value: &str) {
        self.value = CLEAN.clean(value).to_string();
    }
    pub fn get(&self) -> &String {
        &self.value
    }
}

impl Serialize for SafeString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer, {
        serializer.serialize_str(&self.value)
    }
}

struct SafeStringVisitor;

impl<'de> Visitor<'de> for SafeStringVisitor {
    type Value = SafeString;

    fn expecting(&self, formatter:&mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, value: &str) -> Result<SafeString, E>
    where E: serde::de::Error{
        Ok(SafeString::new(value))
    }
}

impl<'de> Deserialize<'de> for SafeString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>, {
        Ok(
            deserializer.deserialize_string(SafeStringVisitor)?
        )
    }
}

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
impl Queryable<Text, diesel::pg::Pg> for SafeString {
    type Row = String;
    fn build(value: Self::Row) -> Self {
        SafeString::new(&value)
    }
}

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
impl Queryable<Text, diesel::sqlite::Sqlite> for SafeString {
    type Row = String;
    fn build(value: Self::Row) -> Self {
        SafeString::new(&value)
    }
}


impl<DB> ToSql<diesel::sql_types::Text, DB> for SafeString
where
    DB: diesel::backend::Backend,
    str: ToSql<diesel::sql_types::Text, DB>, {
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        str::to_sql(&self.value, out)
    }
}


impl Borrow<str> for SafeString {
    fn borrow(&self) -> &str {
        &self.value
    }
}

impl Display for SafeString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Deref for SafeString {
    type Target = str;
    fn deref(&self) -> &str {
        &self.value
    }
}

impl AsRef<str> for SafeString {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

use rocket::request::FromFormValue;
use rocket::http::RawStr;

impl<'v> FromFormValue<'v> for SafeString {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<SafeString, &'v RawStr> {
        let val = String::from_form_value(form_value)?;
        Ok(SafeString::new(&val))
    }
}
