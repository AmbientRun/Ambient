use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

use convert_case::{Case, Casing};
use once_cell::sync::Lazy;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::PackageId;

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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ItemPathBufConstructionError {
    #[error("an identifier path must not be empty")]
    Empty,
    #[error("the item identifier `{identifier}` is not a valid snake_case identifier ({snake_case_error}) or a valid PascalCase identifier ({pascal_case_error})")]
    ItemNotSnakeOrPascalCase {
        identifier: String,
        snake_case_error: String,
        pascal_case_error: String,
    },
    #[error("one of the identifiers in the scope was not snake_case: {reason}")]
    ScopeNotSnakeCase { reason: String },
}
impl From<IdentifierConstructionError<'_>> for ItemPathBufConstructionError {
    fn from(e: IdentifierConstructionError<'_>) -> Self {
        match e {
            IdentifierConstructionError::NotSnakeOrPascalCase {
                identifier,
                snake_case_error,
                pascal_case_error,
            } => Self::ItemNotSnakeOrPascalCase {
                identifier: identifier.to_string(),
                snake_case_error: snake_case_error.to_string(),
                pascal_case_error: pascal_case_error.to_string(),
            },
        }
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
    pub fn new(path: impl Into<String>) -> Result<Self, ItemPathBufConstructionError> {
        Self::new_impl(path.into())
    }

    fn new_impl(path: String) -> Result<Self, ItemPathBufConstructionError> {
        let segments: Vec<_> = path.split("::").filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return Err(ItemPathBufConstructionError::Empty);
        }

        let scope = segments
            .iter()
            .copied()
            .take(segments.len() - 1)
            .map(SnakeCaseIdentifier::new)
            .collect::<Result<_, _>>()
            .map_err(|e| ItemPathBufConstructionError::ScopeNotSnakeCase {
                reason: e.to_string(),
            })?;
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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum IdentifierConstructionError<'a> {
    #[error("the identifier `{identifier}` is not snake_case ({snake_case_error}) or PascalCase ({pascal_case_error})")]
    NotSnakeOrPascalCase {
        identifier: &'a str,
        snake_case_error: SnakeCaseIdentifierConstructionError<'a>,
        pascal_case_error: PascalCaseIdentifierConstructionError<'a>,
    },
}

#[derive(Error, Debug)]
pub enum IdentifierCaseError<'a> {
    #[error("the identifier `{identifier}` is not snake_case")]
    NotSnakeCase { identifier: &'a Identifier },
    #[error("the identifier `{identifier}` is not PascalCase")]
    NotPascalCase { identifier: &'a Identifier },
    #[error("the identifier `{identifier}` is not a package ID")]
    NotPackageId { identifier: &'a Identifier },
}
impl IdentifierCaseError<'_> {
    pub fn to_owned(self) -> IdentifierCaseOwnedError {
        self.into()
    }
}
#[derive(Error, Debug)]
pub enum IdentifierCaseOwnedError {
    #[error("the identifier `{identifier}` is not snake_case")]
    NotSnakeCase { identifier: Identifier },
    #[error("the identifier `{identifier}` is not PascalCase")]
    NotPascalCase { identifier: Identifier },
    #[error("the identifier `{identifier}` is not a package ID")]
    NotPackageId { identifier: Identifier },
}
impl From<IdentifierCaseError<'_>> for IdentifierCaseOwnedError {
    fn from(value: IdentifierCaseError<'_>) -> Self {
        match value {
            IdentifierCaseError::NotSnakeCase { identifier } => Self::NotSnakeCase {
                identifier: identifier.to_owned(),
            },
            IdentifierCaseError::NotPascalCase { identifier } => Self::NotPascalCase {
                identifier: identifier.to_owned(),
            },
            IdentifierCaseError::NotPackageId { identifier } => Self::NotPackageId {
                identifier: identifier.to_owned(),
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Identifier {
    Snake(SnakeCaseIdentifier),
    Pascal(PascalCaseIdentifier),
    PackageId(PackageId),
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
impl From<PackageId> for Identifier {
    fn from(v: PackageId) -> Self {
        Self::PackageId(v)
    }
}
impl Identifier {
    pub fn new(id: &str) -> Result<Self, IdentifierConstructionError> {
        let snake = SnakeCaseIdentifier::new(id);
        let pascal = PascalCaseIdentifier::new(id);

        match (snake, pascal) {
            (Ok(snake), _) => Ok(Self::Snake(snake)),
            (_, Ok(pascal)) => Ok(Self::Pascal(pascal)),
            (Err(snake_case_error), Err(pascal_case_error)) => {
                Err(IdentifierConstructionError::NotSnakeOrPascalCase {
                    identifier: id,
                    snake_case_error,
                    pascal_case_error,
                })
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Identifier::Snake(id) => id.as_str(),
            Identifier::Pascal(id) => id.as_str(),
            Identifier::PackageId(id) => id.as_str(),
        }
    }

    pub fn as_snake(&self) -> Result<&SnakeCaseIdentifier, IdentifierCaseError> {
        match self {
            Self::Snake(v) => Ok(v),
            _ => Err(IdentifierCaseError::NotSnakeCase { identifier: self }),
        }
    }

    pub fn as_pascal(&self) -> Result<&PascalCaseIdentifier, IdentifierCaseError> {
        match self {
            Self::Pascal(v) => Ok(v),
            _ => Err(IdentifierCaseError::NotPascalCase { identifier: self }),
        }
    }

    pub fn as_package_id(&self) -> Result<&PackageId, IdentifierCaseError> {
        match self {
            Self::PackageId(v) => Ok(v),
            _ => Err(IdentifierCaseError::NotPascalCase { identifier: self }),
        }
    }
}
impl Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Snake(id) => Debug::fmt(id, f),
            Identifier::Pascal(id) => Debug::fmt(id, f),
            Identifier::PackageId(id) => Debug::fmt(id, f),
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
            Identifier::PackageId(id) => id.serialize(serializer),
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
            Identifier::PackageId(id) => Display::fmt(id, f),
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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SnakeCaseIdentifierConstructionError<'a> {
    #[error("identifier must not be empty")]
    Empty,
    #[error("identifier `{identifier}` must start with a lowercase ASCII character")]
    NotLowercaseAscii { identifier: &'a str },
    #[error("identifier `{identifier}` is not valid snake-case ASCII")]
    NotSnakeCaseAscii { identifier: &'a str },
    #[error("identifier `{identifier}` is reserved")]
    Reserved { identifier: &'a str },
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SnakeCaseIdentifier(pub(super) String);
impl SnakeCaseIdentifier {
    pub fn new(id: &str) -> Result<Self, SnakeCaseIdentifierConstructionError> {
        Self::validate(id)?;
        Ok(Self(id.to_string()))
    }

    pub fn validate(identifier: &str) -> Result<&str, SnakeCaseIdentifierConstructionError> {
        if identifier.is_empty() {
            return Err(SnakeCaseIdentifierConstructionError::Empty);
        }

        if !identifier.starts_with(|c: char| c.is_ascii_lowercase()) {
            return Err(SnakeCaseIdentifierConstructionError::NotLowercaseAscii { identifier });
        }

        if !identifier
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(SnakeCaseIdentifierConstructionError::NotSnakeCaseAscii { identifier });
        }

        if RESERVED_SNAKE_CASE_IDENTIFIERS.contains(identifier) {
            return Err(SnakeCaseIdentifierConstructionError::Reserved { identifier });
        }

        Ok(identifier)
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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PascalCaseIdentifierConstructionError<'a> {
    #[error("identifier must not be empty")]
    Empty,
    #[error("identifier `{identifier}` must start with an uppercase ASCII character")]
    NotUppercaseAscii { identifier: &'a str },
    #[error("identifier `{identifier}` is not PascalCase ASCII")]
    NotPascalCaseAscii { identifier: &'a str },
    #[error("identifier `{identifier}` is reserved")]
    Reserved { identifier: &'a str },
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PascalCaseIdentifier(pub(super) String);
impl PascalCaseIdentifier {
    pub fn new(id: &str) -> Result<Self, PascalCaseIdentifierConstructionError> {
        Self::validate(id)?;
        Ok(Self(id.to_string()))
    }

    pub fn validate(identifier: &str) -> Result<&str, PascalCaseIdentifierConstructionError> {
        if identifier.is_empty() {
            return Err(PascalCaseIdentifierConstructionError::Empty);
        }

        if !identifier.starts_with(|c: char| c.is_ascii_uppercase()) {
            return Err(PascalCaseIdentifierConstructionError::NotUppercaseAscii { identifier });
        }

        if !identifier
            .chars()
            .all(|c| c.is_ascii_uppercase() | c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return Err(PascalCaseIdentifierConstructionError::NotPascalCaseAscii { identifier });
        }

        if !identifier.is_case(Case::Pascal) {
            return Err(PascalCaseIdentifierConstructionError::NotPascalCaseAscii { identifier });
        }

        if RESERVED_PASCAL_CASE_IDENTIFIERS.contains(identifier) {
            return Err(PascalCaseIdentifierConstructionError::Reserved { identifier });
        }

        Ok(identifier)
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

static RESERVED_SNAKE_CASE_IDENTIFIERS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from_iter([
        // Rust keywords
        "as", "break", "const", "continue", "crate", "else", "extern", "false", "fn", "for", "if",
        "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self",
        "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while",
        "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override",
        "priv", "typeof", "unsized", "virtual", "yield", "try",
        // Identifiers reserved for use by Ambient codegen
        "optional",
    ])
});
static RESERVED_PASCAL_CASE_IDENTIFIERS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| HashSet::from_iter(["Self"]));

#[cfg(test)]
mod tests {
    use crate::{
        Identifier, IdentifierConstructionError, ItemPathBuf, ItemPathBufConstructionError,
        PascalCaseIdentifier, PascalCaseIdentifierConstructionError, SnakeCaseIdentifier,
        SnakeCaseIdentifierConstructionError,
    };

    #[test]
    fn can_construct_identifiers() {
        use Identifier as I;
        use PascalCaseIdentifier as PCI;
        use SnakeCaseIdentifier as SCI;

        assert_eq!(I::new("lol").unwrap(), I::Snake(SCI("lol".to_string())));
        assert_eq!(I::new("Lol").unwrap(), I::Pascal(PCI("Lol".to_string())));
        assert_eq!(I::new("Move").unwrap(), I::Pascal(PCI("Move".to_string())));
        assert_eq!(
            I::new("move"),
            Err(IdentifierConstructionError::NotSnakeOrPascalCase {
                identifier: "move",
                snake_case_error: SnakeCaseIdentifierConstructionError::Reserved {
                    identifier: "move"
                },
                pascal_case_error: PascalCaseIdentifierConstructionError::NotUppercaseAscii {
                    identifier: "move"
                }
            })
        );
    }

    #[test]
    fn can_validate_snake_case_identifiers() {
        use SnakeCaseIdentifier as I;

        assert_eq!(I::new(""), Err(SnakeCaseIdentifierConstructionError::Empty));
        assert_eq!(
            I::new("5asd"),
            Err(SnakeCaseIdentifierConstructionError::NotLowercaseAscii { identifier: "5asd" })
        );
        assert_eq!(
            I::new("_asd"),
            Err(SnakeCaseIdentifierConstructionError::NotLowercaseAscii { identifier: "_asd" })
        );
        assert_eq!(
            I::new("CoolComponent"),
            Err(SnakeCaseIdentifierConstructionError::NotLowercaseAscii {
                identifier: "CoolComponent"
            })
        );
        assert_eq!(
            I::new("mY-COOL-COMPONENT"),
            Err(SnakeCaseIdentifierConstructionError::NotSnakeCaseAscii {
                identifier: "mY-COOL-COMPONENT"
            })
        );
        assert_eq!(
            I::new("cool_component!"),
            Err(SnakeCaseIdentifierConstructionError::NotSnakeCaseAscii {
                identifier: "cool_component!"
            })
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
            Err(SnakeCaseIdentifierConstructionError::NotSnakeCaseAscii {
                identifier: "cool-component"
            })
        );
        assert_eq!(
            I::new("cool-component-c00"),
            Err(SnakeCaseIdentifierConstructionError::NotSnakeCaseAscii {
                identifier: "cool-component-c00"
            })
        );
    }

    #[test]
    fn can_validate_pascal_case_identifiers() {
        use PascalCaseIdentifier as I;

        assert_eq!(
            I::new(""),
            Err(PascalCaseIdentifierConstructionError::Empty)
        );
        assert_eq!(
            I::new("5Asd"),
            Err(PascalCaseIdentifierConstructionError::NotUppercaseAscii { identifier: "5Asd" })
        );
        assert_eq!(
            I::new("_Asd"),
            Err(PascalCaseIdentifierConstructionError::NotUppercaseAscii { identifier: "_Asd" })
        );
        assert_eq!(
            I::new("coolComponent"),
            Err(PascalCaseIdentifierConstructionError::NotUppercaseAscii {
                identifier: "coolComponent"
            })
        );
        assert_eq!(
            I::new("My-Cool-Component"),
            Err(PascalCaseIdentifierConstructionError::NotPascalCaseAscii {
                identifier: "My-Cool-Component"
            })
        );
        assert_eq!(
            I::new("Cool_Component"),
            Err(PascalCaseIdentifierConstructionError::NotPascalCaseAscii {
                identifier: "Cool_Component"
            })
        );
        assert_eq!(I::new("CoolComponent"), Ok(I("CoolComponent".to_string())));
        assert_eq!(
            I::new("CoolComponentC00"),
            Ok(I("CoolComponentC00".to_string()))
        );
        assert_eq!(
            I::new("coolComponent"),
            Err(PascalCaseIdentifierConstructionError::NotUppercaseAscii {
                identifier: "coolComponent"
            })
        );
        assert_eq!(
            I::new("cool-component-c00"),
            Err(PascalCaseIdentifierConstructionError::NotUppercaseAscii {
                identifier: "cool-component-c00"
            })
        );
    }

    #[test]
    fn can_validate_identifier_paths() {
        use ItemPathBuf as IP;
        use PascalCaseIdentifier as PCI;
        use SnakeCaseIdentifier as SCI;

        assert_eq!(IP::new(""), Err(ItemPathBufConstructionError::Empty));

        assert_eq!(
            IP::new("my"),
            Ok(IP {
                scope: vec![],
                item: SCI("my".to_string()).into(),
            })
        );

        assert_eq!(
            IP::new("My Invalid Path"),
            Err(ItemPathBufConstructionError::ItemNotSnakeOrPascalCase {
                identifier: "My Invalid Path".to_string(),
                snake_case_error:
                    "identifier `My Invalid Path` must start with a lowercase ASCII character"
                        .to_string(),
                pascal_case_error: "identifier `My Invalid Path` is not PascalCase ASCII"
                    .to_string(),
            })
        );

        assert_eq!(
            IP::new("MyInvalidPath!"),
            Err(ItemPathBufConstructionError::ItemNotSnakeOrPascalCase {
                identifier: "MyInvalidPath!".to_string(),
                snake_case_error:
                    "identifier `MyInvalidPath!` must start with a lowercase ASCII character"
                        .to_string(),
                pascal_case_error: "identifier `MyInvalidPath!` is not PascalCase ASCII"
                    .to_string(),
            })
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
            Err(ItemPathBufConstructionError::ScopeNotSnakeCase {
                reason: "identifier `CoolComponent00` must start with a lowercase ASCII character"
                    .to_string()
            })
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
