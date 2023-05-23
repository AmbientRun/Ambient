use std::fmt::{Debug, Display};

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemIdentifier<'a> {
    Identifier(&'a Identifier),
    CamelCaseIdentifier(&'a CamelCaseIdentifier),
}
impl<'a> Display for ItemIdentifier<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemIdentifier::Identifier(value) => Display::fmt(value, f),
            ItemIdentifier::CamelCaseIdentifier(value) => Display::fmt(value, f),
        }
    }
}
impl<'a> ToTokens for ItemIdentifier<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ItemIdentifier::Identifier(value) => value.to_tokens(tokens),
            ItemIdentifier::CamelCaseIdentifier(value) => value.to_tokens(tokens),
        }
    }
}
impl ItemIdentifier<'_> {
    pub fn as_identifier(&self) -> Option<&Identifier> {
        match self {
            ItemIdentifier::Identifier(value) => Some(value),
            ItemIdentifier::CamelCaseIdentifier(_) => None,
        }
    }

    pub fn as_camel_case_identifier(&self) -> Option<&CamelCaseIdentifier> {
        match self {
            ItemIdentifier::Identifier(_) => None,
            ItemIdentifier::CamelCaseIdentifier(value) => Some(value),
        }
    }

    pub fn to_owned(&self) -> ItemIdentifierBuf {
        match *self {
            ItemIdentifier::Identifier(value) => ItemIdentifierBuf::Identifier(value.clone()),
            ItemIdentifier::CamelCaseIdentifier(value) => {
                ItemIdentifierBuf::CamelCaseIdentifier(value.clone())
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemIdentifierBuf {
    Identifier(Identifier),
    CamelCaseIdentifier(CamelCaseIdentifier),
}
impl<'a> From<&'a ItemIdentifierBuf> for ItemIdentifier<'a> {
    fn from(value: &'a ItemIdentifierBuf) -> Self {
        match value {
            ItemIdentifierBuf::Identifier(value) => Self::Identifier(value),
            ItemIdentifierBuf::CamelCaseIdentifier(value) => Self::CamelCaseIdentifier(value),
        }
    }
}
impl ItemIdentifierBuf {
    pub fn new(segment: &str) -> Result<Self, &'static str> {
        if let Ok(value) = Identifier::new(segment) {
            return Ok(Self::Identifier(value));
        }

        if let Ok(value) = CamelCaseIdentifier::new(segment) {
            return Ok(Self::CamelCaseIdentifier(value));
        }

        Err("Invalid identifier: is neither valid snake_case nor CamelCase")
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A path to an item (non-owning). Always non-empty.
pub struct ItemPath<'a> {
    scope: &'a [Identifier],
    item: ItemIdentifier<'a>,
}
impl<'a> Display for ItemPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for id in self.item_iter() {
            if !first {
                write!(f, "::")?;
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
        tokens.append_separated(self.item_iter(), quote::quote! {::})
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
    pub fn first(&self) -> ItemIdentifier<'a> {
        match (&self.scope, &self.item) {
            ([], item) => *item,
            (scope, _) => ItemIdentifier::Identifier(&scope[0]),
        }
    }

    pub fn item(&self) -> ItemIdentifier<'a> {
        self.item
    }

    pub fn scope_and_item(&self) -> (&[Identifier], ItemIdentifier<'a>) {
        (&self.scope, self.item)
    }

    /// Returns an iterator over all segments of this path as an iterator of
    /// `ItemIdentifier`s. Used primarily for diagnostics - this will erase
    /// the distinction between `Identifier` and `CamelCaseIdentifier`.
    pub fn item_iter(&self) -> impl Iterator<Item = ItemIdentifier<'a>> {
        self.scope
            .iter()
            .map(ItemIdentifier::Identifier)
            .chain(Some(self.item))
    }

    pub fn to_owned(&self) -> ItemPathBuf {
        ItemPathBuf {
            scope: self.scope.iter().cloned().collect(),
            item: self.item.to_owned(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A path to an item (owning). Always non-empty.
pub struct ItemPathBuf {
    scope: Vec<Identifier>,
    item: ItemIdentifierBuf,
}
impl ItemPathBuf {
    pub fn new(path: impl Into<String>) -> Result<Self, &'static str> {
        Self::new_impl(path.into())
    }

    fn new_impl(path: String) -> Result<Self, &'static str> {
        let segments: Vec<_> = path.split("::").filter(|s| !s.is_empty()).collect();
        if segments.len() == 0 {
            return Err("an identifier path must not be empty");
        }

        let scope = segments
            .iter()
            .copied()
            .take(segments.len() - 1)
            .map(Identifier::new)
            .collect::<Result<_, _>>()?;

        let item = ItemIdentifierBuf::new(segments.last().unwrap())?;

        Ok(Self { scope, item })
    }

    pub fn as_path(&self) -> ItemPath {
        ItemPath {
            scope: &self.scope,
            item: (&self.item).into(),
        }
    }
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
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err("identifier must be snake-case ASCII");
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CamelCaseIdentifier(pub(super) String);
impl CamelCaseIdentifier {
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

        if !id.starts_with(|c: char| c.is_ascii_uppercase()) {
            return Err("identifier must start with a uppercase ASCII character");
        }

        if !id.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("identifier must be camel-case ASCII");
        }

        Ok(id)
    }
}
impl Debug for CamelCaseIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}
impl Serialize for CamelCaseIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.0, serializer)
    }
}
impl<'de> Deserialize<'de> for CamelCaseIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        CamelCaseIdentifier::new_impl(String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}
impl AsRef<str> for CamelCaseIdentifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Display for CamelCaseIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl ToTokens for CamelCaseIdentifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(syn::Ident::new(
            self.as_ref(),
            proc_macro2::Span::call_site(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{CamelCaseIdentifier, Identifier, ItemIdentifierBuf, ItemPathBuf};

    #[test]
    fn can_validate_snake_case_identifiers() {
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
    fn can_validate_camel_case_identifiers() {
        use CamelCaseIdentifier as CI;

        assert_eq!(CI::new(""), Err("identifier must not be empty"));
        assert_eq!(
            CI::new("5asd"),
            Err("identifier must start with a uppercase ASCII character")
        );
        assert_eq!(
            CI::new("_asd"),
            Err("identifier must start with a uppercase ASCII character")
        );
        assert_eq!(
            CI::new("cool_component"),
            Err("identifier must start with a uppercase ASCII character")
        );
        assert_eq!(
            CI::new("mY_COOL_COMPONENT"),
            Err("identifier must start with a uppercase ASCII character")
        );
        assert_eq!(
            CI::new("CoolComponent!"),
            Err("identifier must be camel-case ASCII")
        );
        assert_eq!(
            CI::new("Cool-Component"),
            Err("identifier must be camel-case ASCII")
        );

        assert_eq!(
            CI::new("CoolComponent"),
            Ok(CI("CoolComponent".to_string()))
        );
        assert_eq!(
            CI::new("CoolComponent00"),
            Ok(CI("CoolComponent00".to_string()))
        );
    }

    #[test]
    fn can_validate_identifier_paths() {
        use CamelCaseIdentifier as CCI;
        use Identifier as I;
        use ItemPathBuf as IP;

        assert_eq!(IP::new(""), Err("an identifier path must not be empty"));

        assert_eq!(
            IP::new("my"),
            Ok(IP {
                scope: vec![],
                item: ItemIdentifierBuf::Identifier(I("my".to_string())),
            })
        );

        assert_eq!(
            IP::new("My"),
            Ok(IP {
                scope: vec![],
                item: ItemIdentifierBuf::CamelCaseIdentifier(CCI("My".to_string())),
            })
        );

        assert_eq!(
            IP::new("My Invalid Path"),
            Err("Invalid identifier: is neither valid snake_case nor CamelCase")
        );

        assert_eq!(
            IP::new("MyInvalidPath!"),
            Err("Invalid identifier: is neither valid snake_case nor CamelCase")
        );

        assert_eq!(
            IP::new("my::cool_component_00"),
            Ok(IP {
                scope: vec![I("my".to_string())],
                item: ItemIdentifierBuf::Identifier(I("cool_component_00".to_string())),
            })
        );

        assert_eq!(
            IP::new("my::CoolComponent00"),
            Ok(IP {
                scope: vec![I("my".to_string())],
                item: ItemIdentifierBuf::CamelCaseIdentifier(CCI("CoolComponent00".to_string())),
            })
        );

        assert_eq!(
            IP::new("my::CoolComponent00::lol"),
            Err("identifier must start with a lowercase ASCII character")
        );
    }
}
