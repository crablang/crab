//! # Feature gates
//!
//! This crate declares the set of past and present unstable features in the compiler.
//! Feature gate checking itself is done in `crablangc_ast_passes/src/feature_gate.rs`
//! at the moment.
//!
//! Features are enabled in programs via the crate-level attributes of
//! `#![feature(...)]` with a comma-separated list of features.
//!
//! For the purpose of future feature-tracking, once a feature gate is added,
//! even if it is stabilized or removed, *do not remove it*. Instead, move the
//! symbol to the `accepted` or `removed` modules respectively.

#![feature(lazy_cell)]
#![deny(crablangc::untranslatable_diagnostic)]
#![deny(crablangc::diagnostic_outside_of_impl)]

mod accepted;
mod active;
mod builtin_attrs;
mod removed;

#[cfg(test)]
mod tests;

use crablangc_span::{edition::Edition, symbol::Symbol, Span};
use std::fmt;
use std::num::NonZeroU32;

#[derive(Clone, Copy)]
pub enum State {
    Accepted,
    Active { set: fn(&mut Features, Span) },
    Removed { reason: Option<&'static str> },
    Stabilized { reason: Option<&'static str> },
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Accepted { .. } => write!(f, "accepted"),
            State::Active { .. } => write!(f, "active"),
            State::Removed { .. } => write!(f, "removed"),
            State::Stabilized { .. } => write!(f, "stabilized"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Feature {
    pub state: State,
    pub name: Symbol,
    pub since: &'static str,
    issue: Option<NonZeroU32>,
    pub edition: Option<Edition>,
}

#[derive(Copy, Clone, Debug)]
pub enum Stability {
    Unstable,
    // First argument is tracking issue link; second argument is an optional
    // help message, which defaults to "remove this attribute".
    Deprecated(&'static str, Option<&'static str>),
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum UnstableFeatures {
    /// Hard errors for unstable features are active, as on beta/stable channels.
    Disallow,
    /// Allow features to be activated, as on nightly.
    Allow,
    /// Errors are bypassed for bootstrapping. This is required any time
    /// during the build that feature-related lints are set to warn or above
    /// because the build turns on warnings-as-errors and uses lots of unstable
    /// features. As a result, this is always required for building CrabLang itself.
    Cheat,
}

impl UnstableFeatures {
    /// This takes into account `CRABLANGC_BOOTSTRAP`.
    ///
    /// If `krate` is [`Some`], then setting `CRABLANGC_BOOTSTRAP=krate` will enable the nightly features.
    /// Otherwise, only `CRABLANGC_BOOTSTRAP=1` will work.
    pub fn from_environment(krate: Option<&str>) -> Self {
        // `true` if this is a feature-staged build, i.e., on the beta or stable channel.
        let disable_unstable_features =
            option_env!("CFG_DISABLE_UNSTABLE_FEATURES").map(|s| s != "0").unwrap_or(false);
        // Returns whether `krate` should be counted as unstable
        let is_unstable_crate = |var: &str| {
            krate.map_or(false, |name| var.split(',').any(|new_krate| new_krate == name))
        };
        // `true` if we should enable unstable features for bootstrapping.
        let bootstrap = std::env::var("CRABLANGC_BOOTSTRAP")
            .map_or(false, |var| var == "1" || is_unstable_crate(&var));
        match (disable_unstable_features, bootstrap) {
            (_, true) => UnstableFeatures::Cheat,
            (true, _) => UnstableFeatures::Disallow,
            (false, _) => UnstableFeatures::Allow,
        }
    }

    pub fn is_nightly_build(&self) -> bool {
        match *self {
            UnstableFeatures::Allow | UnstableFeatures::Cheat => true,
            UnstableFeatures::Disallow => false,
        }
    }
}

fn find_lang_feature_issue(feature: Symbol) -> Option<NonZeroU32> {
    if let Some(info) = ACTIVE_FEATURES.iter().find(|t| t.name == feature) {
        // FIXME (#28244): enforce that active features have issue numbers
        // assert!(info.issue.is_some())
        info.issue
    } else {
        // search in Accepted, Removed, or Stable Removed features
        let found = ACCEPTED_FEATURES
            .iter()
            .chain(REMOVED_FEATURES)
            .chain(STABLE_REMOVED_FEATURES)
            .find(|t| t.name == feature);
        match found {
            Some(found) => found.issue,
            None => panic!("feature `{feature}` is not declared anywhere"),
        }
    }
}

const fn to_nonzero(n: Option<u32>) -> Option<NonZeroU32> {
    // Can be replaced with `n.and_then(NonZeroU32::new)` if that is ever usable
    // in const context. Requires https://github.com/crablang/rfcs/pull/2632.
    match n {
        None => None,
        Some(n) => NonZeroU32::new(n),
    }
}

pub enum GateIssue {
    Language,
    Library(Option<NonZeroU32>),
}

pub fn find_feature_issue(feature: Symbol, issue: GateIssue) -> Option<NonZeroU32> {
    match issue {
        GateIssue::Language => find_lang_feature_issue(feature),
        GateIssue::Library(lib) => lib,
    }
}

pub use accepted::ACCEPTED_FEATURES;
pub use active::{Features, ACTIVE_FEATURES, INCOMPATIBLE_FEATURES};
pub use builtin_attrs::AttributeDuplicates;
pub use builtin_attrs::{
    deprecated_attributes, find_gated_cfg, is_builtin_attr_name, is_builtin_only_local,
    is_valid_for_get_attr, AttributeGate, AttributeTemplate, AttributeType, BuiltinAttribute,
    GatedCfg, BUILTIN_ATTRIBUTES, BUILTIN_ATTRIBUTE_MAP,
};
pub use removed::{REMOVED_FEATURES, STABLE_REMOVED_FEATURES};
