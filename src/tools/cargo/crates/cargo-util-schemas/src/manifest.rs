//! `Cargo.toml` / Manifest schema definition
//!
//! ## Style
//!
//! - Fields duplicated for an alias will have an accessor with the primary field's name
//! - Keys that exist for bookkeeping but don't correspond to the schema have a `_` prefix

use std::collections::BTreeMap;
use std::fmt::{self, Display, Write};
use std::path::PathBuf;
use std::str;

use serde::de::{self, IntoDeserializer as _, Unexpected};
use serde::ser;
use serde::{Deserialize, Serialize};
use serde_untagged::UntaggedEnumVisitor;

use crate::core::PackageIdSpec;
use crate::core::PartialVersion;
use crate::core::PartialVersionError;
use crate::restricted_names;

pub use crate::restricted_names::NameValidationError;

/// This type is used to deserialize `Cargo.toml` files.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct TomlManifest {
    pub cargo_features: Option<Vec<String>>,
    pub package: Option<Box<TomlPackage>>,
    pub project: Option<Box<TomlPackage>>,
    pub profile: Option<TomlProfiles>,
    pub lib: Option<TomlLibTarget>,
    pub bin: Option<Vec<TomlBinTarget>>,
    pub example: Option<Vec<TomlExampleTarget>>,
    pub test: Option<Vec<TomlTestTarget>>,
    pub bench: Option<Vec<TomlTestTarget>>,
    pub dependencies: Option<BTreeMap<PackageName, InheritableDependency>>,
    pub dev_dependencies: Option<BTreeMap<PackageName, InheritableDependency>>,
    #[serde(rename = "dev_dependencies")]
    pub dev_dependencies2: Option<BTreeMap<PackageName, InheritableDependency>>,
    pub build_dependencies: Option<BTreeMap<PackageName, InheritableDependency>>,
    #[serde(rename = "build_dependencies")]
    pub build_dependencies2: Option<BTreeMap<PackageName, InheritableDependency>>,
    pub features: Option<BTreeMap<FeatureName, Vec<String>>>,
    pub target: Option<BTreeMap<String, TomlPlatform>>,
    pub replace: Option<BTreeMap<String, TomlDependency>>,
    pub patch: Option<BTreeMap<String, BTreeMap<PackageName, TomlDependency>>>,
    pub workspace: Option<TomlWorkspace>,
    pub badges: Option<InheritableBtreeMap>,
    pub lints: Option<InheritableLints>,
    // when adding new fields, be sure to check whether `requires_package` should disallow them
}

impl TomlManifest {
    pub fn requires_package(&self) -> impl Iterator<Item = &'static str> {
        [
            self.lib.as_ref().map(|_| "lib"),
            self.bin.as_ref().map(|_| "bin"),
            self.example.as_ref().map(|_| "example"),
            self.test.as_ref().map(|_| "test"),
            self.bench.as_ref().map(|_| "bench"),
            self.dependencies.as_ref().map(|_| "dependencies"),
            self.dev_dependencies().as_ref().map(|_| "dev-dependencies"),
            self.build_dependencies()
                .as_ref()
                .map(|_| "build-dependencies"),
            self.features.as_ref().map(|_| "features"),
            self.target.as_ref().map(|_| "target"),
            self.badges.as_ref().map(|_| "badges"),
            self.lints.as_ref().map(|_| "lints"),
        ]
        .into_iter()
        .flatten()
    }

    pub fn has_profiles(&self) -> bool {
        self.profile.is_some()
    }

    pub fn package(&self) -> Option<&Box<TomlPackage>> {
        self.package.as_ref().or(self.project.as_ref())
    }

    pub fn dev_dependencies(&self) -> Option<&BTreeMap<PackageName, InheritableDependency>> {
        self.dev_dependencies
            .as_ref()
            .or(self.dev_dependencies2.as_ref())
    }

    pub fn build_dependencies(&self) -> Option<&BTreeMap<PackageName, InheritableDependency>> {
        self.build_dependencies
            .as_ref()
            .or(self.build_dependencies2.as_ref())
    }

    pub fn features(&self) -> Option<&BTreeMap<FeatureName, Vec<String>>> {
        self.features.as_ref()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TomlWorkspace {
    pub members: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub default_members: Option<Vec<String>>,
    pub resolver: Option<String>,
    pub metadata: Option<toml::Value>,

    // Properties that can be inherited by members.
    pub package: Option<InheritablePackage>,
    pub dependencies: Option<BTreeMap<PackageName, TomlDependency>>,
    pub lints: Option<TomlLints>,
}

/// A group of fields that are inheritable by members of the workspace
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct InheritablePackage {
    pub version: Option<semver::Version>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub readme: Option<StringOrBool>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<String>,
    pub publish: Option<VecStringOrBool>,
    pub edition: Option<String>,
    pub badges: Option<BTreeMap<String, BTreeMap<String, String>>>,
    pub exclude: Option<Vec<String>>,
    pub include: Option<Vec<String>>,
    pub rust_version: Option<RustVersion>,
}

/// Represents the `package`/`project` sections of a `Cargo.toml`.
///
/// Note that the order of the fields matters, since this is the order they
/// are serialized to a TOML file. For example, you cannot have values after
/// the field `metadata`, since it is a table and values cannot appear after
/// tables.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TomlPackage {
    pub edition: Option<InheritableString>,
    pub rust_version: Option<InheritableRustVersion>,
    pub name: PackageName,
    pub version: Option<InheritableSemverVersion>,
    pub authors: Option<InheritableVecString>,
    pub build: Option<StringOrBool>,
    pub metabuild: Option<StringOrVec>,
    pub default_target: Option<String>,
    pub forced_target: Option<String>,
    pub links: Option<String>,
    pub exclude: Option<InheritableVecString>,
    pub include: Option<InheritableVecString>,
    pub publish: Option<InheritableVecStringOrBool>,
    pub workspace: Option<String>,
    pub im_a_teapot: Option<bool>,
    pub autobins: Option<bool>,
    pub autoexamples: Option<bool>,
    pub autotests: Option<bool>,
    pub autobenches: Option<bool>,
    pub default_run: Option<String>,

    // Package metadata.
    pub description: Option<InheritableString>,
    pub homepage: Option<InheritableString>,
    pub documentation: Option<InheritableString>,
    pub readme: Option<InheritableStringOrBool>,
    pub keywords: Option<InheritableVecString>,
    pub categories: Option<InheritableVecString>,
    pub license: Option<InheritableString>,
    pub license_file: Option<InheritableString>,
    pub repository: Option<InheritableString>,
    pub resolver: Option<String>,

    pub metadata: Option<toml::Value>,

    /// Provide a helpful error message for a common user error.
    #[serde(rename = "cargo-features", skip_serializing)]
    pub _invalid_cargo_features: Option<InvalidCargoFeatures>,
}

/// An enum that allows for inheriting keys from a workspace in a Cargo.toml.
#[derive(Serialize, Copy, Clone, Debug)]
#[serde(untagged)]
pub enum InheritableField<T> {
    /// The type that that is used when not inheriting from a workspace.
    Value(T),
    /// The type when inheriting from a workspace.
    Inherit(TomlInheritedField),
}

impl<T> InheritableField<T> {
    pub fn as_value(&self) -> Option<&T> {
        match self {
            InheritableField::Inherit(_) => None,
            InheritableField::Value(defined) => Some(defined),
        }
    }
}

//. This already has a `Deserialize` impl from version_trim_whitespace
pub type InheritableSemverVersion = InheritableField<semver::Version>;
impl<'de> de::Deserialize<'de> for InheritableSemverVersion {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .expecting("SemVer version")
            .string(
                |value| match value.trim().parse().map_err(de::Error::custom) {
                    Ok(parsed) => Ok(InheritableField::Value(parsed)),
                    Err(e) => Err(e),
                },
            )
            .map(|value| value.deserialize().map(InheritableField::Inherit))
            .deserialize(d)
    }
}

pub type InheritableString = InheritableField<String>;
impl<'de> de::Deserialize<'de> for InheritableString {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = InheritableString;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                f.write_str("a string or workspace")
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(InheritableString::Value(value))
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mvd = de::value::MapAccessDeserializer::new(map);
                TomlInheritedField::deserialize(mvd).map(InheritableField::Inherit)
            }
        }

        d.deserialize_any(Visitor)
    }
}

pub type InheritableRustVersion = InheritableField<RustVersion>;
impl<'de> de::Deserialize<'de> for InheritableRustVersion {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = InheritableRustVersion;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                f.write_str("a semver or workspace")
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = value.parse::<RustVersion>().map_err(|e| E::custom(e))?;
                Ok(InheritableRustVersion::Value(value))
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mvd = de::value::MapAccessDeserializer::new(map);
                TomlInheritedField::deserialize(mvd).map(InheritableField::Inherit)
            }
        }

        d.deserialize_any(Visitor)
    }
}

pub type InheritableVecString = InheritableField<Vec<String>>;
impl<'de> de::Deserialize<'de> for InheritableVecString {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = InheritableVecString;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                f.write_str("a vector of strings or workspace")
            }
            fn visit_seq<A>(self, v: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let seq = de::value::SeqAccessDeserializer::new(v);
                Vec::deserialize(seq).map(InheritableField::Value)
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mvd = de::value::MapAccessDeserializer::new(map);
                TomlInheritedField::deserialize(mvd).map(InheritableField::Inherit)
            }
        }

        d.deserialize_any(Visitor)
    }
}

pub type InheritableStringOrBool = InheritableField<StringOrBool>;
impl<'de> de::Deserialize<'de> for InheritableStringOrBool {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = InheritableStringOrBool;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                f.write_str("a string, a bool, or workspace")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let b = de::value::BoolDeserializer::new(v);
                StringOrBool::deserialize(b).map(InheritableField::Value)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let string = de::value::StringDeserializer::new(v);
                StringOrBool::deserialize(string).map(InheritableField::Value)
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mvd = de::value::MapAccessDeserializer::new(map);
                TomlInheritedField::deserialize(mvd).map(InheritableField::Inherit)
            }
        }

        d.deserialize_any(Visitor)
    }
}

pub type InheritableVecStringOrBool = InheritableField<VecStringOrBool>;
impl<'de> de::Deserialize<'de> for InheritableVecStringOrBool {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = InheritableVecStringOrBool;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                f.write_str("a boolean, a vector of strings, or workspace")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let b = de::value::BoolDeserializer::new(v);
                VecStringOrBool::deserialize(b).map(InheritableField::Value)
            }

            fn visit_seq<A>(self, v: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let seq = de::value::SeqAccessDeserializer::new(v);
                VecStringOrBool::deserialize(seq).map(InheritableField::Value)
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mvd = de::value::MapAccessDeserializer::new(map);
                TomlInheritedField::deserialize(mvd).map(InheritableField::Inherit)
            }
        }

        d.deserialize_any(Visitor)
    }
}

pub type InheritableBtreeMap = InheritableField<BTreeMap<String, BTreeMap<String, String>>>;

impl<'de> de::Deserialize<'de> for InheritableBtreeMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let value = serde_value::Value::deserialize(deserializer)?;

        if let Ok(w) = TomlInheritedField::deserialize(
            serde_value::ValueDeserializer::<D::Error>::new(value.clone()),
        ) {
            return Ok(InheritableField::Inherit(w));
        }
        BTreeMap::deserialize(serde_value::ValueDeserializer::<D::Error>::new(value))
            .map(InheritableField::Value)
    }
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TomlInheritedField {
    workspace: WorkspaceValue,
}

impl TomlInheritedField {
    pub fn new() -> Self {
        TomlInheritedField {
            workspace: WorkspaceValue,
        }
    }
}

impl Default for TomlInheritedField {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug)]
#[serde(try_from = "bool")]
#[serde(into = "bool")]
struct WorkspaceValue;

impl TryFrom<bool> for WorkspaceValue {
    type Error = String;
    fn try_from(other: bool) -> Result<WorkspaceValue, Self::Error> {
        if other {
            Ok(WorkspaceValue)
        } else {
            Err("`workspace` cannot be false".to_owned())
        }
    }
}

impl From<WorkspaceValue> for bool {
    fn from(_: WorkspaceValue) -> bool {
        true
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum InheritableDependency {
    /// The type that that is used when not inheriting from a workspace.
    Value(TomlDependency),
    /// The type when inheriting from a workspace.
    Inherit(TomlInheritedDependency),
}

impl InheritableDependency {
    pub fn unused_keys(&self) -> Vec<String> {
        match self {
            InheritableDependency::Value(d) => d.unused_keys(),
            InheritableDependency::Inherit(w) => w._unused_keys.keys().cloned().collect(),
        }
    }
}

impl<'de> de::Deserialize<'de> for InheritableDependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let value = serde_value::Value::deserialize(deserializer)?;

        if let Ok(w) = TomlInheritedDependency::deserialize(serde_value::ValueDeserializer::<
            D::Error,
        >::new(value.clone()))
        {
            return if w.workspace {
                Ok(InheritableDependency::Inherit(w))
            } else {
                Err(de::Error::custom("`workspace` cannot be false"))
            };
        }
        TomlDependency::deserialize(serde_value::ValueDeserializer::<D::Error>::new(value))
            .map(InheritableDependency::Value)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TomlInheritedDependency {
    pub workspace: bool,
    pub features: Option<Vec<String>>,
    pub default_features: Option<bool>,
    #[serde(rename = "default_features")]
    pub default_features2: Option<bool>,
    pub optional: Option<bool>,
    pub public: Option<bool>,

    /// This is here to provide a way to see the "unused manifest keys" when deserializing
    #[serde(skip_serializing)]
    #[serde(flatten)]
    pub _unused_keys: BTreeMap<String, toml::Value>,
}

impl TomlInheritedDependency {
    pub fn default_features(&self) -> Option<bool> {
        self.default_features.or(self.default_features2)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum TomlDependency<P: Clone = String> {
    /// In the simple format, only a version is specified, eg.
    /// `package = "<version>"`
    Simple(String),
    /// The simple format is equivalent to a detailed dependency
    /// specifying only a version, eg.
    /// `package = { version = "<version>" }`
    Detailed(TomlDetailedDependency<P>),
}

impl TomlDependency {
    pub fn is_version_specified(&self) -> bool {
        match self {
            TomlDependency::Detailed(d) => d.version.is_some(),
            TomlDependency::Simple(..) => true,
        }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            TomlDependency::Detailed(d) => d.optional.unwrap_or(false),
            TomlDependency::Simple(..) => false,
        }
    }

    pub fn is_public(&self) -> bool {
        match self {
            TomlDependency::Detailed(d) => d.public.unwrap_or(false),
            TomlDependency::Simple(..) => false,
        }
    }

    pub fn unused_keys(&self) -> Vec<String> {
        match self {
            TomlDependency::Simple(_) => vec![],
            TomlDependency::Detailed(detailed) => detailed._unused_keys.keys().cloned().collect(),
        }
    }
}

impl<'de, P: Deserialize<'de> + Clone> de::Deserialize<'de> for TomlDependency<P> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .expecting(
                "a version string like \"0.9.8\" or a \
                     detailed dependency like { version = \"0.9.8\" }",
            )
            .string(|value| Ok(TomlDependency::Simple(value.to_owned())))
            .map(|value| value.deserialize().map(TomlDependency::Detailed))
            .deserialize(deserializer)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TomlDetailedDependency<P: Clone = String> {
    pub version: Option<String>,
    pub registry: Option<RegistryName>,
    /// The URL of the `registry` field.
    /// This is an internal implementation detail. When Cargo creates a
    /// package, it replaces `registry` with `registry-index` so that the
    /// manifest contains the correct URL. All users won't have the same
    /// registry names configured, so Cargo can't rely on just the name for
    /// crates published by other users.
    pub registry_index: Option<String>,
    // `path` is relative to the file it appears in. If that's a `Cargo.toml`, it'll be relative to
    // that TOML file, and if it's a `.cargo/config` file, it'll be relative to that file.
    pub path: Option<P>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub rev: Option<String>,
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub default_features: Option<bool>,
    #[serde(rename = "default_features")]
    pub default_features2: Option<bool>,
    pub package: Option<PackageName>,
    pub public: Option<bool>,

    /// One or more of `bin`, `cdylib`, `staticlib`, `bin:<name>`.
    pub artifact: Option<StringOrVec>,
    /// If set, the artifact should also be a dependency
    pub lib: Option<bool>,
    /// A platform name, like `x86_64-apple-darwin`
    pub target: Option<String>,

    /// This is here to provide a way to see the "unused manifest keys" when deserializing
    #[serde(skip_serializing)]
    #[serde(flatten)]
    pub _unused_keys: BTreeMap<String, toml::Value>,
}

impl<P: Clone> TomlDetailedDependency<P> {
    pub fn default_features(&self) -> Option<bool> {
        self.default_features.or(self.default_features2)
    }
}

// Explicit implementation so we avoid pulling in P: Default
impl<P: Clone> Default for TomlDetailedDependency<P> {
    fn default() -> Self {
        Self {
            version: Default::default(),
            registry: Default::default(),
            registry_index: Default::default(),
            path: Default::default(),
            git: Default::default(),
            branch: Default::default(),
            tag: Default::default(),
            rev: Default::default(),
            features: Default::default(),
            optional: Default::default(),
            default_features: Default::default(),
            default_features2: Default::default(),
            package: Default::default(),
            public: Default::default(),
            artifact: Default::default(),
            lib: Default::default(),
            target: Default::default(),
            _unused_keys: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct TomlProfiles(pub BTreeMap<ProfileName, TomlProfile>);

impl TomlProfiles {
    pub fn get_all(&self) -> &BTreeMap<ProfileName, TomlProfile> {
        &self.0
    }

    pub fn get(&self, name: &str) -> Option<&TomlProfile> {
        self.0.get(name)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, Eq, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct TomlProfile {
    pub opt_level: Option<TomlOptLevel>,
    pub lto: Option<StringOrBool>,
    pub codegen_backend: Option<String>,
    pub codegen_units: Option<u32>,
    pub debug: Option<TomlDebugInfo>,
    pub split_debuginfo: Option<String>,
    pub debug_assertions: Option<bool>,
    pub rpath: Option<bool>,
    pub panic: Option<String>,
    pub overflow_checks: Option<bool>,
    pub incremental: Option<bool>,
    pub dir_name: Option<String>,
    pub inherits: Option<String>,
    pub strip: Option<StringOrBool>,
    // Note that `rustflags` is used for the cargo-feature `profile_rustflags`
    pub rustflags: Option<Vec<String>>,
    // These two fields must be last because they are sub-tables, and TOML
    // requires all non-tables to be listed first.
    pub package: Option<BTreeMap<ProfilePackageSpec, TomlProfile>>,
    pub build_override: Option<Box<TomlProfile>>,
    /// Unstable feature `-Ztrim-paths`.
    pub trim_paths: Option<TomlTrimPaths>,
}

impl TomlProfile {
    /// Overwrite self's values with the given profile.
    pub fn merge(&mut self, profile: &Self) {
        if let Some(v) = &profile.opt_level {
            self.opt_level = Some(v.clone());
        }

        if let Some(v) = &profile.lto {
            self.lto = Some(v.clone());
        }

        if let Some(v) = &profile.codegen_backend {
            self.codegen_backend = Some(v.clone());
        }

        if let Some(v) = profile.codegen_units {
            self.codegen_units = Some(v);
        }

        if let Some(v) = profile.debug {
            self.debug = Some(v);
        }

        if let Some(v) = profile.debug_assertions {
            self.debug_assertions = Some(v);
        }

        if let Some(v) = &profile.split_debuginfo {
            self.split_debuginfo = Some(v.clone());
        }

        if let Some(v) = profile.rpath {
            self.rpath = Some(v);
        }

        if let Some(v) = &profile.panic {
            self.panic = Some(v.clone());
        }

        if let Some(v) = profile.overflow_checks {
            self.overflow_checks = Some(v);
        }

        if let Some(v) = profile.incremental {
            self.incremental = Some(v);
        }

        if let Some(v) = &profile.rustflags {
            self.rustflags = Some(v.clone());
        }

        if let Some(other_package) = &profile.package {
            match &mut self.package {
                Some(self_package) => {
                    for (spec, other_pkg_profile) in other_package {
                        match self_package.get_mut(spec) {
                            Some(p) => p.merge(other_pkg_profile),
                            None => {
                                self_package.insert(spec.clone(), other_pkg_profile.clone());
                            }
                        }
                    }
                }
                None => self.package = Some(other_package.clone()),
            }
        }

        if let Some(other_bo) = &profile.build_override {
            match &mut self.build_override {
                Some(self_bo) => self_bo.merge(other_bo),
                None => self.build_override = Some(other_bo.clone()),
            }
        }

        if let Some(v) = &profile.inherits {
            self.inherits = Some(v.clone());
        }

        if let Some(v) = &profile.dir_name {
            self.dir_name = Some(v.clone());
        }

        if let Some(v) = &profile.strip {
            self.strip = Some(v.clone());
        }

        if let Some(v) = &profile.trim_paths {
            self.trim_paths = Some(v.clone())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum ProfilePackageSpec {
    Spec(PackageIdSpec),
    All,
}

impl fmt::Display for ProfilePackageSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfilePackageSpec::Spec(spec) => spec.fmt(f),
            ProfilePackageSpec::All => f.write_str("*"),
        }
    }
}

impl ser::Serialize for ProfilePackageSpec {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_string().serialize(s)
    }
}

impl<'de> de::Deserialize<'de> for ProfilePackageSpec {
    fn deserialize<D>(d: D) -> Result<ProfilePackageSpec, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let string = String::deserialize(d)?;
        if string == "*" {
            Ok(ProfilePackageSpec::All)
        } else {
            PackageIdSpec::parse(&string)
                .map_err(de::Error::custom)
                .map(ProfilePackageSpec::Spec)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TomlOptLevel(pub String);

impl ser::Serialize for TomlOptLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self.0.parse::<u32>() {
            Ok(n) => n.serialize(serializer),
            Err(_) => self.0.serialize(serializer),
        }
    }
}

impl<'de> de::Deserialize<'de> for TomlOptLevel {
    fn deserialize<D>(d: D) -> Result<TomlOptLevel, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use serde::de::Error as _;
        UntaggedEnumVisitor::new()
            .expecting("an optimization level")
            .i64(|value| Ok(TomlOptLevel(value.to_string())))
            .string(|value| {
                if value == "s" || value == "z" {
                    Ok(TomlOptLevel(value.to_string()))
                } else {
                    Err(serde_untagged::de::Error::custom(format!(
                        "must be `0`, `1`, `2`, `3`, `s` or `z`, \
                         but found the string: \"{}\"",
                        value
                    )))
                }
            })
            .deserialize(d)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TomlDebugInfo {
    None,
    LineDirectivesOnly,
    LineTablesOnly,
    Limited,
    Full,
}

impl Display for TomlDebugInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TomlDebugInfo::None => f.write_char('0'),
            TomlDebugInfo::Limited => f.write_char('1'),
            TomlDebugInfo::Full => f.write_char('2'),
            TomlDebugInfo::LineDirectivesOnly => f.write_str("line-directives-only"),
            TomlDebugInfo::LineTablesOnly => f.write_str("line-tables-only"),
        }
    }
}

impl ser::Serialize for TomlDebugInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            Self::None => 0.serialize(serializer),
            Self::LineDirectivesOnly => "line-directives-only".serialize(serializer),
            Self::LineTablesOnly => "line-tables-only".serialize(serializer),
            Self::Limited => 1.serialize(serializer),
            Self::Full => 2.serialize(serializer),
        }
    }
}

impl<'de> de::Deserialize<'de> for TomlDebugInfo {
    fn deserialize<D>(d: D) -> Result<TomlDebugInfo, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use serde::de::Error as _;
        let expecting = "a boolean, 0, 1, 2, \"line-tables-only\", or \"line-directives-only\"";
        UntaggedEnumVisitor::new()
            .expecting(expecting)
            .bool(|value| {
                Ok(if value {
                    TomlDebugInfo::Full
                } else {
                    TomlDebugInfo::None
                })
            })
            .i64(|value| {
                let debuginfo = match value {
                    0 => TomlDebugInfo::None,
                    1 => TomlDebugInfo::Limited,
                    2 => TomlDebugInfo::Full,
                    _ => {
                        return Err(serde_untagged::de::Error::invalid_value(
                            Unexpected::Signed(value),
                            &expecting,
                        ))
                    }
                };
                Ok(debuginfo)
            })
            .string(|value| {
                let debuginfo = match value {
                    "none" => TomlDebugInfo::None,
                    "limited" => TomlDebugInfo::Limited,
                    "full" => TomlDebugInfo::Full,
                    "line-directives-only" => TomlDebugInfo::LineDirectivesOnly,
                    "line-tables-only" => TomlDebugInfo::LineTablesOnly,
                    _ => {
                        return Err(serde_untagged::de::Error::invalid_value(
                            Unexpected::Str(value),
                            &expecting,
                        ))
                    }
                };
                Ok(debuginfo)
            })
            .deserialize(d)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize)]
#[serde(untagged, rename_all = "kebab-case")]
pub enum TomlTrimPaths {
    Values(Vec<TomlTrimPathsValue>),
    All,
}

impl TomlTrimPaths {
    pub fn none() -> Self {
        TomlTrimPaths::Values(Vec::new())
    }

    pub fn is_none(&self) -> bool {
        match self {
            TomlTrimPaths::Values(v) => v.is_empty(),
            TomlTrimPaths::All => false,
        }
    }
}

impl<'de> de::Deserialize<'de> for TomlTrimPaths {
    fn deserialize<D>(d: D) -> Result<TomlTrimPaths, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use serde::de::Error as _;
        let expecting = r#"a boolean, "none", "diagnostics", "macro", "object", "all", or an array with these options"#;
        UntaggedEnumVisitor::new()
            .expecting(expecting)
            .bool(|value| {
                Ok(if value {
                    TomlTrimPaths::All
                } else {
                    TomlTrimPaths::none()
                })
            })
            .string(|v| match v {
                "none" => Ok(TomlTrimPaths::none()),
                "all" => Ok(TomlTrimPaths::All),
                v => {
                    let d = v.into_deserializer();
                    let err = |_: D::Error| {
                        serde_untagged::de::Error::custom(format!("expected {expecting}"))
                    };
                    TomlTrimPathsValue::deserialize(d)
                        .map_err(err)
                        .map(|v| v.into())
                }
            })
            .seq(|seq| {
                let seq: Vec<String> = seq.deserialize()?;
                let seq: Vec<_> = seq
                    .into_iter()
                    .map(|s| TomlTrimPathsValue::deserialize(s.into_deserializer()))
                    .collect::<Result<_, _>>()?;
                Ok(seq.into())
            })
            .deserialize(d)
    }
}

impl fmt::Display for TomlTrimPaths {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TomlTrimPaths::All => write!(f, "all"),
            TomlTrimPaths::Values(v) if v.is_empty() => write!(f, "none"),
            TomlTrimPaths::Values(v) => {
                let mut iter = v.iter();
                if let Some(value) = iter.next() {
                    write!(f, "{value}")?;
                }
                for value in iter {
                    write!(f, ",{value}")?;
                }
                Ok(())
            }
        }
    }
}

impl From<TomlTrimPathsValue> for TomlTrimPaths {
    fn from(value: TomlTrimPathsValue) -> Self {
        TomlTrimPaths::Values(vec![value])
    }
}

impl From<Vec<TomlTrimPathsValue>> for TomlTrimPaths {
    fn from(value: Vec<TomlTrimPathsValue>) -> Self {
        TomlTrimPaths::Values(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TomlTrimPathsValue {
    Diagnostics,
    Macro,
    Object,
}

impl TomlTrimPathsValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            TomlTrimPathsValue::Diagnostics => "diagnostics",
            TomlTrimPathsValue::Macro => "macro",
            TomlTrimPathsValue::Object => "object",
        }
    }
}

impl fmt::Display for TomlTrimPathsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub type TomlLibTarget = TomlTarget;
pub type TomlBinTarget = TomlTarget;
pub type TomlExampleTarget = TomlTarget;
pub type TomlTestTarget = TomlTarget;
pub type TomlBenchTarget = TomlTarget;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TomlTarget {
    pub name: Option<String>,

    // The intention was to only accept `crate-type` here but historical
    // versions of Cargo also accepted `crate_type`, so look for both.
    pub crate_type: Option<Vec<String>>,
    #[serde(rename = "crate_type")]
    pub crate_type2: Option<Vec<String>>,

    pub path: Option<PathValue>,
    // Note that `filename` is used for the cargo-feature `different_binary_name`
    pub filename: Option<String>,
    pub test: Option<bool>,
    pub doctest: Option<bool>,
    pub bench: Option<bool>,
    pub doc: Option<bool>,
    pub plugin: Option<bool>,
    pub doc_scrape_examples: Option<bool>,
    pub proc_macro: Option<bool>,
    #[serde(rename = "proc_macro")]
    pub proc_macro2: Option<bool>,
    pub harness: Option<bool>,
    pub required_features: Option<Vec<String>>,
    pub edition: Option<String>,
}

impl TomlTarget {
    pub fn new() -> TomlTarget {
        TomlTarget::default()
    }

    pub fn proc_macro(&self) -> Option<bool> {
        self.proc_macro.or(self.proc_macro2).or_else(|| {
            if let Some(types) = self.crate_types() {
                if types.contains(&"proc-macro".to_string()) {
                    return Some(true);
                }
            }
            None
        })
    }

    pub fn crate_types(&self) -> Option<&Vec<String>> {
        self.crate_type
            .as_ref()
            .or_else(|| self.crate_type2.as_ref())
    }
}

macro_rules! str_newtype {
    ($name:ident) => {
        /// Verified string newtype
        #[derive(Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(transparent)]
        pub struct $name<T: AsRef<str> = String>(T);

        impl<T: AsRef<str>> $name<T> {
            pub fn into_inner(self) -> T {
                self.0
            }
        }

        impl<T: AsRef<str>> AsRef<str> for $name<T> {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl<T: AsRef<str>> std::ops::Deref for $name<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T: AsRef<str>> std::borrow::Borrow<str> for $name<T> {
            fn borrow(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl<'a> std::str::FromStr for $name<String> {
            type Err = restricted_names::NameValidationError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value.to_owned())
            }
        }

        impl<'de, T: AsRef<str> + serde::Deserialize<'de>> serde::Deserialize<'de> for $name<T> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let inner = T::deserialize(deserializer)?;
                Self::new(inner).map_err(serde::de::Error::custom)
            }
        }

        impl<T: AsRef<str>> Display for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.as_ref().fmt(f)
            }
        }
    };
}

str_newtype!(PackageName);

impl<T: AsRef<str>> PackageName<T> {
    /// Validated package name
    pub fn new(name: T) -> Result<Self, NameValidationError> {
        restricted_names::validate_package_name(name.as_ref(), "package name")?;
        Ok(Self(name))
    }
}

impl PackageName {
    /// Coerce a value to be a validate package name
    ///
    /// Replaces invalid values with `placeholder`
    pub fn sanitize(name: impl AsRef<str>, placeholder: char) -> Self {
        PackageName(restricted_names::sanitize_package_name(
            name.as_ref(),
            placeholder,
        ))
    }
}

str_newtype!(RegistryName);

impl<T: AsRef<str>> RegistryName<T> {
    /// Validated registry name
    pub fn new(name: T) -> Result<Self, NameValidationError> {
        restricted_names::validate_package_name(name.as_ref(), "registry name")?;
        Ok(Self(name))
    }
}

str_newtype!(ProfileName);

impl<T: AsRef<str>> ProfileName<T> {
    /// Validated profile name
    pub fn new(name: T) -> Result<Self, NameValidationError> {
        restricted_names::validate_profile_name(name.as_ref())?;
        Ok(Self(name))
    }
}

str_newtype!(FeatureName);

impl<T: AsRef<str>> FeatureName<T> {
    /// Validated feature name
    pub fn new(name: T) -> Result<Self, NameValidationError> {
        restricted_names::validate_feature_name(name.as_ref())?;
        Ok(Self(name))
    }
}

/// Corresponds to a `target` entry, but `TomlTarget` is already used.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TomlPlatform {
    pub dependencies: Option<BTreeMap<PackageName, InheritableDependency>>,
    pub build_dependencies: Option<BTreeMap<PackageName, InheritableDependency>>,
    #[serde(rename = "build_dependencies")]
    pub build_dependencies2: Option<BTreeMap<PackageName, InheritableDependency>>,
    pub dev_dependencies: Option<BTreeMap<PackageName, InheritableDependency>>,
    #[serde(rename = "dev_dependencies")]
    pub dev_dependencies2: Option<BTreeMap<PackageName, InheritableDependency>>,
}

impl TomlPlatform {
    pub fn dev_dependencies(&self) -> Option<&BTreeMap<PackageName, InheritableDependency>> {
        self.dev_dependencies
            .as_ref()
            .or(self.dev_dependencies2.as_ref())
    }

    pub fn build_dependencies(&self) -> Option<&BTreeMap<PackageName, InheritableDependency>> {
        self.build_dependencies
            .as_ref()
            .or(self.build_dependencies2.as_ref())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct InheritableLints {
    #[serde(skip_serializing_if = "is_false")]
    pub workspace: bool,
    #[serde(flatten)]
    pub lints: TomlLints,
}

fn is_false(b: &bool) -> bool {
    !b
}

impl<'de> Deserialize<'de> for InheritableLints {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct InheritableLintsVisitor;

        impl<'de> de::Visitor<'de> for InheritableLintsVisitor {
            // The type that our Visitor is going to produce.
            type Value = InheritableLints;

            // Format a message stating what data this Visitor expects to receive.
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a lints table")
            }

            // Deserialize MyMap from an abstract "map" provided by the
            // Deserializer. The MapAccess input is a callback provided by
            // the Deserializer to let us see each entry in the map.
            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut lints = TomlLints::new();
                let mut workspace = false;

                // While there are entries remaining in the input, add them
                // into our map.
                while let Some(key) = access.next_key()? {
                    if key == "workspace" {
                        workspace = match access.next_value()? {
                            Some(WorkspaceValue) => true,
                            None => false,
                        };
                    } else {
                        let value = access.next_value()?;
                        lints.insert(key, value);
                    }
                }

                Ok(InheritableLints { workspace, lints })
            }
        }

        deserializer.deserialize_map(InheritableLintsVisitor)
    }
}

pub type TomlLints = BTreeMap<String, TomlToolLints>;

pub type TomlToolLints = BTreeMap<String, TomlLint>;

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum TomlLint {
    Level(TomlLintLevel),
    Config(TomlLintConfig),
}

impl<'de> Deserialize<'de> for TomlLint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .string(|string| {
                TomlLintLevel::deserialize(string.into_deserializer()).map(TomlLint::Level)
            })
            .map(|map| map.deserialize().map(TomlLint::Config))
            .deserialize(deserializer)
    }
}

impl TomlLint {
    pub fn level(&self) -> TomlLintLevel {
        match self {
            Self::Level(level) => *level,
            Self::Config(config) => config.level,
        }
    }

    pub fn priority(&self) -> i8 {
        match self {
            Self::Level(_) => 0,
            Self::Config(config) => config.priority,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct TomlLintConfig {
    pub level: TomlLintLevel,
    #[serde(default)]
    pub priority: i8,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum TomlLintLevel {
    Forbid,
    Deny,
    Warn,
    Allow,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug, serde::Serialize)]
#[serde(transparent)]
pub struct RustVersion(PartialVersion);

impl std::ops::Deref for RustVersion {
    type Target = PartialVersion;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::str::FromStr for RustVersion {
    type Err = RustVersionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let partial = value.parse::<PartialVersion>();
        let partial = partial.map_err(RustVersionErrorKind::PartialVersion)?;
        if partial.pre.is_some() {
            return Err(RustVersionErrorKind::Prerelease.into());
        }
        if partial.build.is_some() {
            return Err(RustVersionErrorKind::BuildMetadata.into());
        }
        Ok(Self(partial))
    }
}

impl<'de> serde::Deserialize<'de> for RustVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .expecting("SemVer version")
            .string(|value| value.parse().map_err(serde::de::Error::custom))
            .deserialize(deserializer)
    }
}

impl Display for RustVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Error parsing a [`RustVersion`].
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct RustVersionError(#[from] RustVersionErrorKind);

/// Non-public error kind for [`RustVersionError`].
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
enum RustVersionErrorKind {
    #[error("unexpected prerelease field, expected a version like \"1.32\"")]
    Prerelease,

    #[error("unexpected build field, expected a version like \"1.32\"")]
    BuildMetadata,

    #[error(transparent)]
    PartialVersion(#[from] PartialVersionError),
}

#[derive(Copy, Clone, Debug)]
pub struct InvalidCargoFeatures {}

impl<'de> de::Deserialize<'de> for InvalidCargoFeatures {
    fn deserialize<D>(_d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use serde::de::Error as _;

        Err(D::Error::custom(
            "the field `cargo-features` should be set at the top of Cargo.toml before any tables",
        ))
    }
}

/// A StringOrVec can be parsed from either a TOML string or array,
/// but is always stored as a vector.
#[derive(Clone, Debug, Serialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct StringOrVec(pub Vec<String>);

impl StringOrVec {
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, String> {
        self.0.iter()
    }
}

impl<'de> de::Deserialize<'de> for StringOrVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .expecting("string or list of strings")
            .string(|value| Ok(StringOrVec(vec![value.to_owned()])))
            .seq(|value| value.deserialize().map(StringOrVec))
            .deserialize(deserializer)
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum StringOrBool {
    String(String),
    Bool(bool),
}

impl<'de> Deserialize<'de> for StringOrBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .bool(|b| Ok(StringOrBool::Bool(b)))
            .string(|s| Ok(StringOrBool::String(s.to_owned())))
            .deserialize(deserializer)
    }
}

#[derive(PartialEq, Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum VecStringOrBool {
    VecString(Vec<String>),
    Bool(bool),
}

impl<'de> de::Deserialize<'de> for VecStringOrBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        UntaggedEnumVisitor::new()
            .expecting("a boolean or vector of strings")
            .bool(|value| Ok(VecStringOrBool::Bool(value)))
            .seq(|value| value.deserialize().map(VecStringOrBool::VecString))
            .deserialize(deserializer)
    }
}

#[derive(Clone)]
pub struct PathValue(pub PathBuf);

impl fmt::Debug for PathValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ser::Serialize for PathValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for PathValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(PathValue(String::deserialize(deserializer)?.into()))
    }
}
