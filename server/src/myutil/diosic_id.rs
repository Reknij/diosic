use serde::{Serialize, Deserialize, de::Visitor};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiosicID {
    content: String
}

struct IDVisitor;

impl<'de> Visitor<'de> for IDVisitor {
    type Value = String;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        Ok(v.to_owned())
    }
}

impl Serialize for DiosicID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(self.content.as_str())
    }
}

impl<'de> Deserialize<'de> for DiosicID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let content = deserializer.deserialize_string(IDVisitor)?;

        Ok(Self {
            content
        })
    }
}

impl<T> From<T> for DiosicID where T: ToString {
    fn from(n: T) -> Self {
        DiosicID { content: n.to_string() }
    }
}

impl DiosicID {
    pub fn as_str(&self)-> &str {
        &self.content.as_str()
    }
}