use std::{fmt::Display, ops::Deref};

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentifierPath<'a>(pub &'a [Identifier]);
impl<'a> IdentifierPath<'a> {}
impl<'a> Display for IdentifierPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for id in self.0 {
            if !first {
                write!(f, "::")?;
            }

            write!(f, "{}", id.0)?;
            first = false;
        }

        Ok(())
    }
}
impl<'a> Deref for IdentifierPath<'a> {
    type Target = [Identifier];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a> ToTokens for IdentifierPath<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_separated(self.0.iter(), quote::quote! {::})
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdentifierPathBuf(pub(super) Vec<Identifier>);
impl IdentifierPathBuf {
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn new(path: impl Into<String>) -> Result<Self, &'static str> {
        Self::new_impl(path.into())
    }

    fn new_impl(path: String) -> Result<Self, &'static str> {
        Ok(Self(
            path.split("::")
                .map(Identifier::new)
                .collect::<Result<_, _>>()?,
        ))
    }

    pub fn push(&mut self, value: Identifier) {
        self.0.push(value)
    }

    pub fn as_path(&self) -> IdentifierPath {
        IdentifierPath(&self.0)
    }
}
impl Display for IdentifierPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.as_path(), f)
    }
}
impl<'de> Deserialize<'de> for IdentifierPathBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        IdentifierPathBuf::new_impl(String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}
impl Deref for IdentifierPathBuf {
    type Target = [Identifier];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl FromIterator<Identifier> for IdentifierPathBuf {
    fn from_iter<T: IntoIterator<Item = Identifier>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
impl ToTokens for IdentifierPathBuf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_path().to_tokens(tokens)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Identifier(pub(super) String);
impl Identifier {
    pub fn new(id: impl Into<String>) -> Result<Self, &'static str> {
        Self::new_impl(id.into())
    }

    fn new_impl(id: String) -> Result<Self, &'static str> {
        Self::validate(&id)?;
        Ok(Self(id))
    }

    pub fn validate(id: &str) -> Result<&str, &'static str> {
        if id.is_empty() {
            return Err("identifier must not be empty");
        }

        if !id.starts_with(|c: char| c.is_ascii_lowercase()) {
            return Err("identifier must start with a lowercase ASCII character");
        }

        if !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err("identifier must be snake-case ASCII");
        }

        Ok(id)
    }
}
impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.0, serializer)
    }
}
impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Identifier::new_impl(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl ToTokens for Identifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(syn::Ident::new(
            self.as_ref(),
            proc_macro2::Span::call_site(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Identifier, IdentifierPathBuf};

    #[test]
    fn can_validate_identifiers() {
        use Identifier as I;

        assert_eq!(I::new(""), Err("identifier must not be empty"));
        assert_eq!(
            I::new("5asd"),
            Err("identifier must start with a lowercase ASCII character")
        );
        assert_eq!(
            I::new("_asd"),
            Err("identifier must start with a lowercase ASCII character")
        );
        assert_eq!(
            I::new("mY_COOL_COMPONENT"),
            Err("identifier must be snake-case ASCII")
        );
        assert_eq!(
            I::new("cool_component!"),
            Err("identifier must be snake-case ASCII")
        );
        assert_eq!(
            I::new("cool-component"),
            Err("identifier must be snake-case ASCII")
        );

        assert_eq!(
            I::new("cool_component"),
            Ok(I("cool_component".to_string()))
        );
        assert_eq!(
            I::new("cool_component_00"),
            Ok(I("cool_component_00".to_string()))
        );
    }

    #[test]
    fn can_validate_identifier_paths() {
        use Identifier as I;
        use IdentifierPathBuf as IP;

        assert_eq!(
            IP::new("my::cool_component_00"),
            Ok(IP(vec![
                I("my".to_string()),
                I("cool_component_00".to_string())
            ]))
        );
    }
}
