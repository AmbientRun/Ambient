use std::fmt::{Debug, Display};

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A path to an item (non-owning). Always non-empty.
pub struct ItemPath<'a> {
    scope: &'a [Identifier],
    item: &'a Identifier,
}
impl<'a> Display for ItemPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for id in self.iter() {
            if !first {
                write!(f, "/")?;
            }

            write!(f, "{}", id)?;
            first = false;
        }

        Ok(())
    }
}
impl<'a> Debug for ItemPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
impl<'a> ToTokens for ItemPath<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_separated(self.iter(), quote::quote! {::})
    }
}
impl Serialize for ItemPath<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
impl<'a> ItemPath<'a> {
    pub fn first(&self) -> &'a Identifier {
        match (self.scope, self.item) {
            ([], item) => item,
            (scope, _) => &scope[0],
        }
    }

    pub fn item(&self) -> &'a Identifier {
        self.item
    }

    pub fn scope_and_item(&self) -> (&[Identifier], &Identifier) {
        (self.scope, self.item)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Identifier> {
        self.scope.iter().chain(Some(self.item))
    }

    pub fn to_owned(&self) -> ItemPathBuf {
        ItemPathBuf {
            scope: self.scope.to_vec(),
            item: self.item.to_owned(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A path to an item (owning). Always non-empty.
pub struct ItemPathBuf {
    scope: Vec<Identifier>,
    item: Identifier,
}
impl Display for ItemPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.as_path(), f)
    }
}
impl Debug for ItemPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.as_path(), f)
    }
}
impl<'de> Deserialize<'de> for ItemPathBuf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        ItemPathBuf::new_impl(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
impl Serialize for ItemPathBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_path().serialize(serializer)
    }
}
impl ToTokens for ItemPathBuf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.as_path().to_tokens(tokens)
    }
}
impl From<ItemPath<'_>> for ItemPathBuf {
    fn from(path: ItemPath<'_>) -> Self {
        path.to_owned()
    }
}
impl From<Identifier> for ItemPathBuf {
    fn from(item: Identifier) -> Self {
        Self {
            scope: Vec::new(),
            item,
        }
    }
}
impl ItemPathBuf {
    pub fn new(path: impl Into<String>) -> Result<Self, &'static str> {
        Self::new_impl(path.into())
    }

    fn new_impl(path: String) -> Result<Self, &'static str> {
        let segments: Vec<_> = path.split("/").filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return Err("an identifier path must not be empty");
        }

        let scope = segments
            .iter()
            .copied()
            .take(segments.len() - 1)
            .map(Identifier::new)
            .collect::<Result<_, _>>()?;
        let item = Identifier::new(*segments.last().unwrap())?;

        Ok(Self { scope, item })
    }

    pub fn as_path(&self) -> ItemPath {
        ItemPath {
            scope: &self.scope,
            item: &self.item,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err("identifier must be kebab-case ASCII");
        }

        if !id
            .split('-')
            .all(|c| c.chars().next().map(|c| c.is_ascii_lowercase()) == Some(true))
        {
            return Err(
                "each segment of an identifier must start with a lowercase ASCII character",
            );
        }

        Ok(id)
    }
}
impl Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
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
    use crate::{Identifier, ItemPathBuf};

    #[test]
    fn can_validate_kebab_case_identifiers() {
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
            I::new("CoolComponent"),
            Err("identifier must start with a lowercase ASCII character")
        );
        assert_eq!(
            I::new("mY_COOL_COMPONENT"),
            Err("identifier must be kebab-case ASCII")
        );
        assert_eq!(
            I::new("cool_component!"),
            Err("identifier must be kebab-case ASCII")
        );
        assert_eq!(
            I::new("cool_component"),
            Err("identifier must be kebab-case ASCII")
        );
        assert_eq!(
            I::new("cool_component_00"),
            Err("identifier must be kebab-case ASCII")
        );
        assert_eq!(
            I::new("cool-5-component"),
            Err("each segment of an identifier must start with a lowercase ASCII character")
        );

        assert_eq!(
            I::new("cool-component"),
            Ok(I("cool-component".to_string()))
        );
        assert_eq!(
            I::new("cool-component-c00"),
            Ok(I("cool-component-c00".to_string()))
        );
    }

    #[test]
    fn can_validate_identifier_paths() {
        use Identifier as I;
        use ItemPathBuf as IP;

        assert_eq!(IP::new(""), Err("an identifier path must not be empty"));

        assert_eq!(
            IP::new("my"),
            Ok(IP {
                scope: vec![],
                item: I("my".to_string()),
            })
        );

        assert_eq!(
            IP::new("My Invalid Path"),
            Err("identifier must start with a lowercase ASCII character")
        );

        assert_eq!(
            IP::new("MyInvalidPath!"),
            Err("identifier must start with a lowercase ASCII character")
        );

        assert_eq!(
            IP::new("my/CoolComponent00"),
            Err("identifier must start with a lowercase ASCII character")
        );

        assert_eq!(
            IP::new("My"),
            Err("identifier must start with a lowercase ASCII character")
        );

        assert_eq!(
            IP::new("my/CoolComponent00/lol"),
            Err("identifier must start with a lowercase ASCII character")
        );

        assert_eq!(
            IP::new("my/cool-component-c00"),
            Ok(IP {
                scope: vec![I("my".to_string())],
                item: I("cool-component-c00".to_string()),
            })
        );
    }
}
