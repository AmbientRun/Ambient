use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

use convert_case::{Case, Casing};
use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A path to an item (non-owning). Always non-empty.
pub struct ItemPath<'a> {
    scope: &'a [SnakeCaseIdentifier],
    item: &'a Identifier,
}
impl<'a> Display for ItemPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for id in self.str_iter() {
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
        tokens.append_separated(self.token_iter(), quote::quote! {::})
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
    pub fn item(&self) -> &'a Identifier {
        self.item
    }

    pub fn scope_and_item(&self) -> (&[SnakeCaseIdentifier], &Identifier) {
        (self.scope, self.item)
    }

    /// Returns the first element of the path and the rest of the path.
    /// If the scope is empty, returns `None`.
    ///
    /// Useful for traversing into a hierarchy.
    pub fn split_first(&self) -> Option<(&SnakeCaseIdentifier, ItemPath)> {
        let (first, scope) = self.scope.split_first()?;
        Some((
            first,
            ItemPath {
                scope,
                item: self.item,
            },
        ))
    }

    /// Returns the path without the first element.
    pub fn without_first(&self) -> Option<ItemPath> {
        self.split_first().map(|(_, rest)| rest)
    }

    pub fn to_owned(&self) -> ItemPathBuf {
        ItemPathBuf {
            scope: self.scope.to_vec(),
            item: self.item.to_owned(),
        }
    }

    pub fn str_iter(&self) -> impl Iterator<Item = &str> {
        self.scope
            .iter()
            .map(|id| id.as_str())
            .chain(std::iter::once(self.item.as_str()))
    }

    pub fn token_iter(&self) -> impl Iterator<Item = &dyn ToTokens> {
        self.scope
            .iter()
            .map(|id| id as &dyn ToTokens)
            .chain(std::iter::once(self.item as &dyn ToTokens))
    }

    /// If `item` is a snake-case identifier, this will return a full iter including the scope.
    ///
    /// Otherwise, it will return None.
    pub fn scope_iter(&self) -> Option<impl Iterator<Item = &SnakeCaseIdentifier>> {
        let last = self.item.as_snake().ok()?;
        Some(self.scope.iter().chain(std::iter::once(last)))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A path to an item (owning). Always non-empty.
pub struct ItemPathBuf {
    scope: Vec<SnakeCaseIdentifier>,
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
        let segments: Vec<_> = path.split("::").filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return Err("an identifier path must not be empty");
        }

        let scope = segments
            .iter()
            .copied()
            .take(segments.len() - 1)
            .map(SnakeCaseIdentifier::new)
            .collect::<Result<_, _>>()?;
        let item = Identifier::new(segments.last().unwrap())?;

        Ok(Self { scope, item })
    }

    pub fn as_path(&self) -> ItemPath {
        ItemPath {
            scope: &self.scope,
            item: &self.item,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Identifier {
    Snake(SnakeCaseIdentifier),
    Pascal(PascalCaseIdentifier),
}
impl From<SnakeCaseIdentifier> for Identifier {
    fn from(v: SnakeCaseIdentifier) -> Self {
        Self::Snake(v)
    }
}
impl From<PascalCaseIdentifier> for Identifier {
    fn from(v: PascalCaseIdentifier) -> Self {
        Self::Pascal(v)
    }
}
impl Identifier {
    pub fn new(id: &str) -> Result<Self, &'static str> {
        if let Ok(id) = SnakeCaseIdentifier::new(id) {
            return Ok(Self::Snake(id));
        }
        if let Ok(id) = PascalCaseIdentifier::new(id) {
            return Ok(Self::Pascal(id));
        }
        Err("identifier must be snake-case or PascalCase ASCII")
    }

    pub fn as_str(&self) -> &str {
        match self {
            Identifier::Snake(id) => id.as_str(),
            Identifier::Pascal(id) => id.as_str(),
        }
    }

    pub fn as_snake(&self) -> anyhow::Result<&SnakeCaseIdentifier> {
        match self {
            Self::Snake(v) => Ok(v),
            _ => anyhow::bail!("the identifier {} is not snake_case", self),
        }
    }

    pub fn as_pascal(&self) -> anyhow::Result<&PascalCaseIdentifier> {
        match self {
            Self::Pascal(v) => Ok(v),
            _ => anyhow::bail!("the identifier {} is not PascalCase", self),
        }
    }
}
impl Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Snake(id) => Debug::fmt(id, f),
            Identifier::Pascal(id) => Debug::fmt(id, f),
        }
    }
}
impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Identifier::Snake(id) => id.serialize(serializer),
            Identifier::Pascal(id) => id.serialize(serializer),
        }
    }
}
impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Identifier::new(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Snake(id) => Display::fmt(id, f),
            Identifier::Pascal(id) => Display::fmt(id, f),
        }
    }
}
impl ToTokens for Identifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(syn::Ident::new(
            self.as_str(),
            proc_macro2::Span::call_site(),
        ))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SnakeCaseIdentifier(pub(super) String);
impl SnakeCaseIdentifier {
    pub fn new(id: &str) -> Result<Self, &'static str> {
        Self::validate(id)?;
        Ok(Self(id.to_string()))
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

        if BANNED_SNAKE_CASE_IDENTIFIERS.contains(id) {
            return Err("identifier is banned");
        }

        Ok(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl Debug for SnakeCaseIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}
impl Serialize for SnakeCaseIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.0, serializer)
    }
}
impl<'de> Deserialize<'de> for SnakeCaseIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SnakeCaseIdentifier::new(&String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}
impl Display for SnakeCaseIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl ToTokens for SnakeCaseIdentifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(syn::Ident::new(
            self.as_str(),
            proc_macro2::Span::call_site(),
        ))
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PascalCaseIdentifier(pub(super) String);
impl PascalCaseIdentifier {
    pub fn new(id: &str) -> Result<Self, &'static str> {
        Self::validate(id)?;
        Ok(Self(id.to_string()))
    }

    pub fn validate(id: &str) -> Result<&str, &'static str> {
        if id.is_empty() {
            return Err("identifier must not be empty");
        }

        if !id.starts_with(|c: char| c.is_ascii_uppercase()) {
            return Err("identifier must start with an uppercase ASCII character");
        }

        if !id
            .chars()
            .all(|c| c.is_ascii_uppercase() | c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return Err("identifier must be PascalCase ASCII");
        }

        if !id.is_case(Case::Pascal) {
            return Err("identifier must be PascalCase ASCII");
        }

        if BANNED_PASCAL_CASE_IDENTIFIERS.contains(id) {
            return Err("identifier is banned");
        }

        Ok(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl Debug for PascalCaseIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}
impl Serialize for PascalCaseIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.0, serializer)
    }
}
impl<'de> Deserialize<'de> for PascalCaseIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        PascalCaseIdentifier::new(&String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}
impl Display for PascalCaseIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl ToTokens for PascalCaseIdentifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(syn::Ident::new(
            self.as_str(),
            proc_macro2::Span::call_site(),
        ))
    }
}

const BANNED_IDENTIFIERS: &[(&str, &str)] = &[
    //
    // Ambient
    //
    ("ember", "Ember"),
    //
    // Rust
    //
    ("as", "As"),
    ("break", "Break"),
    ("const", "Const"),
    ("continue", "Continue"),
    ("crate", "Crate"),
    ("else", "Else"),
    ("extern", "Extern"),
    ("false", "False"),
    ("fn", "Fn"),
    ("for", "For"),
    ("if", "If"),
    ("impl", "Impl"),
    ("in", "In"),
    ("let", "Let"),
    ("loop", "Loop"),
    ("match", "Match"),
    ("mod", "Mod"),
    ("move", "Move"),
    ("mut", "Mut"),
    ("pub", "Pub"),
    ("ref", "Ref"),
    ("return", "Return"),
    ("self", "Self"),
    ("static", "Static"),
    ("struct", "Struct"),
    ("super", "Super"),
    ("trait", "Trait"),
    ("true", "True"),
    ("type", "Type"),
    ("unsafe", "Unsafe"),
    ("use", "Use"),
    ("where", "Where"),
    ("while", "While"),
    ("async", "Async"),
    ("await", "Await"),
    ("dyn", "Dyn"),
    ("abstract", "Abstract"),
    ("become", "Become"),
    ("box", "Box"),
    ("do", "Do"),
    ("final", "Final"),
    ("macro", "Macro"),
    ("override", "Override"),
    ("priv", "Priv"),
    ("typeof", "Typeof"),
    ("unsized", "Unsized"),
    ("virtual", "Virtual"),
    ("yield", "Yield"),
    ("try", "Try"),
];

static BANNED_SNAKE_CASE_IDENTIFIERS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| HashSet::from_iter(BANNED_IDENTIFIERS.iter().map(|p| p.0)));
static BANNED_PASCAL_CASE_IDENTIFIERS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| HashSet::from_iter(BANNED_IDENTIFIERS.iter().map(|p| p.1)));

#[cfg(test)]
mod tests {
    use crate::{ItemPathBuf, PascalCaseIdentifier, SnakeCaseIdentifier};

    #[test]
    fn can_validate_snake_case_identifiers() {
        use SnakeCaseIdentifier as I;

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
            I::new("mY-COOL-COMPONENT"),
            Err("identifier must be snake-case ASCII")
        );
        assert_eq!(
            I::new("cool_component!"),
            Err("identifier must be snake-case ASCII")
        );
        assert_eq!(
            I::new("cool_component"),
            Ok(I("cool_component".to_string()))
        );
        assert_eq!(
            I::new("cool_component_c00"),
            Ok(I("cool_component_c00".to_string()))
        );

        assert_eq!(
            I::new("cool-component"),
            Err("identifier must be snake-case ASCII")
        );
        assert_eq!(
            I::new("cool-component-c00"),
            Err("identifier must be snake-case ASCII")
        );
    }

    #[test]
    fn can_validate_pascal_case_identifiers() {
        use PascalCaseIdentifier as I;

        assert_eq!(I::new(""), Err("identifier must not be empty"));
        assert_eq!(
            I::new("5Asd"),
            Err("identifier must start with an uppercase ASCII character")
        );
        assert_eq!(
            I::new("_Asd"),
            Err("identifier must start with an uppercase ASCII character")
        );
        assert_eq!(
            I::new("coolComponent"),
            Err("identifier must start with an uppercase ASCII character")
        );
        assert_eq!(
            I::new("My-Cool-Component"),
            Err("identifier must be PascalCase ASCII")
        );
        assert_eq!(
            I::new("Cool_Component"),
            Err("identifier must be PascalCase ASCII")
        );
        assert_eq!(I::new("CoolComponent"), Ok(I("CoolComponent".to_string())));
        assert_eq!(
            I::new("CoolComponentC00"),
            Ok(I("CoolComponentC00".to_string()))
        );
        assert_eq!(
            I::new("coolComponent"),
            Err("identifier must start with an uppercase ASCII character")
        );
        assert_eq!(
            I::new("cool-component-c00"),
            Err("identifier must start with an uppercase ASCII character")
        );
    }

    #[test]
    fn can_validate_identifier_paths() {
        use ItemPathBuf as IP;
        use PascalCaseIdentifier as PCI;
        use SnakeCaseIdentifier as SCI;

        assert_eq!(IP::new(""), Err("an identifier path must not be empty"));

        assert_eq!(
            IP::new("my"),
            Ok(IP {
                scope: vec![],
                item: SCI("my".to_string()).into(),
            })
        );

        assert_eq!(
            IP::new("My Invalid Path"),
            Err("identifier must be snake-case or PascalCase ASCII")
        );

        assert_eq!(
            IP::new("MyInvalidPath!"),
            Err("identifier must be snake-case or PascalCase ASCII")
        );

        assert_eq!(
            IP::new("my::CoolComponent00"),
            Ok(IP {
                scope: vec![SCI("my".to_string())],
                item: PCI("CoolComponent00".to_string()).into(),
            })
        );

        assert_eq!(
            IP::new("My"),
            Ok(IP {
                scope: vec![],
                item: PCI("My".to_string()).into(),
            })
        );

        assert_eq!(
            IP::new("my::CoolComponent00::lol"),
            Err("identifier must start with a lowercase ASCII character")
        );

        assert_eq!(
            IP::new("my::cool_component_c00"),
            Ok(IP {
                scope: vec![SCI("my".to_string())],
                item: SCI("cool_component_c00".to_string()).into(),
            })
        );
    }
}
