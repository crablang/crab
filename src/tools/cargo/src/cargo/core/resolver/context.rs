use super::dep_cache::RegistryQueryer;
use super::errors::ActivateResult;
use super::types::{ConflictMap, ConflictReason, FeaturesSet, ResolveOpts};
use super::RequestedFeatures;
use crate::core::{Dependency, PackageId, SourceId, Summary};
use crate::util::interning::InternedString;
use crate::util::Graph;
use anyhow::format_err;
use std::collections::HashMap;
use std::num::NonZeroU64;
use tracing::debug;

// A `Context` is basically a bunch of local resolution information which is
// kept around for all `BacktrackFrame` instances. As a result, this runs the
// risk of being cloned *a lot* so we want to make this as cheap to clone as
// possible.
#[derive(Clone)]
pub struct Context {
    pub age: ContextAge,
    pub activations: Activations,
    /// list the features that are activated for each package
    pub resolve_features: im_rc::HashMap<PackageId, FeaturesSet>,
    /// get the package that will be linking to a native library by its links attribute
    pub links: im_rc::HashMap<InternedString, PackageId>,

    /// a way to look up for a package in activations what packages required it
    /// and all of the exact deps that it fulfilled.
    pub parents: Graph<PackageId, im_rc::HashSet<Dependency>>,
}

/// When backtracking it can be useful to know how far back to go.
/// The `ContextAge` of a `Context` is a monotonically increasing counter of the number
/// of decisions made to get to this state.
/// Several structures store the `ContextAge` when it was added,
/// to be used in `find_candidate` for backtracking.
pub type ContextAge = usize;

/// Find the activated version of a crate based on the name, source, and semver compatibility.
/// By storing this in a hash map we ensure that there is only one
/// semver compatible version of each crate.
/// This all so stores the `ContextAge`.
pub type ActivationsKey = (InternedString, SourceId, SemverCompatibility);
pub type Activations = im_rc::HashMap<ActivationsKey, (Summary, ContextAge)>;

/// A type that represents when cargo treats two Versions as compatible.
/// Versions `a` and `b` are compatible if their left-most nonzero digit is the
/// same.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub enum SemverCompatibility {
    Major(NonZeroU64),
    Minor(NonZeroU64),
    Patch(u64),
}

impl From<&semver::Version> for SemverCompatibility {
    fn from(ver: &semver::Version) -> Self {
        if let Some(m) = NonZeroU64::new(ver.major) {
            return SemverCompatibility::Major(m);
        }
        if let Some(m) = NonZeroU64::new(ver.minor) {
            return SemverCompatibility::Minor(m);
        }
        SemverCompatibility::Patch(ver.patch)
    }
}

impl PackageId {
    pub fn as_activations_key(self) -> ActivationsKey {
        (self.name(), self.source_id(), self.version().into())
    }
}

impl Context {
    pub fn new() -> Context {
        Context {
            age: 0,
            resolve_features: im_rc::HashMap::new(),
            links: im_rc::HashMap::new(),
            parents: Graph::new(),
            activations: im_rc::HashMap::new(),
        }
    }

    /// Activate this summary by inserting it into our list of known activations.
    ///
    /// The `parent` passed in here is the parent summary/dependency edge which
    /// cased `summary` to get activated. This may not be present for the root
    /// crate, for example.
    ///
    /// Returns `true` if this summary with the given features is already activated.
    pub fn flag_activated(
        &mut self,
        summary: &Summary,
        opts: &ResolveOpts,
        parent: Option<(&Summary, &Dependency)>,
    ) -> ActivateResult<bool> {
        let id = summary.package_id();
        let age: ContextAge = self.age;
        match self.activations.entry(id.as_activations_key()) {
            im_rc::hashmap::Entry::Occupied(o) => {
                debug_assert_eq!(
                    &o.get().0,
                    summary,
                    "cargo does not allow two semver compatible versions"
                );
            }
            im_rc::hashmap::Entry::Vacant(v) => {
                if let Some(link) = summary.links() {
                    if self.links.insert(link, id).is_some() {
                        return Err(format_err!(
                            "Attempting to resolve a dependency with more than \
                             one crate with links={}.\nThis will not build as \
                             is. Consider rebuilding the .lock file.",
                            &*link
                        )
                        .into());
                    }
                }
                v.insert((summary.clone(), age));

                // If we've got a parent dependency which activated us, *and*
                // the dependency has a different source id listed than the
                // `summary` itself, then things get interesting. This basically
                // means that a `[patch]` was used to augment `dep.source_id()`
                // with `summary`.
                //
                // In this scenario we want to consider the activation key, as
                // viewed from the perspective of `dep.source_id()`, as being
                // fulfilled. This means that we need to add a second entry in
                // the activations map for the source that was patched, in
                // addition to the source of the actual `summary` itself.
                //
                // Without this it would be possible to have both 1.0.0 and
                // 1.1.0 "from crates.io" in a dependency graph if one of those
                // versions came from a `[patch]` source.
                if let Some((_, dep)) = parent {
                    if dep.source_id() != id.source_id() {
                        let key = (id.name(), dep.source_id(), id.version().into());
                        let prev = self.activations.insert(key, (summary.clone(), age));
                        if let Some((previous_summary, _)) = prev {
                            return Err(
                                (previous_summary.package_id(), ConflictReason::Semver).into()
                            );
                        }
                    }
                }

                return Ok(false);
            }
        }
        debug!("checking if {} is already activated", summary.package_id());
        match &opts.features {
            // This returns `false` for CliFeatures just for simplicity. It
            // would take a bit of work to compare since they are not in the
            // same format as DepFeatures (and that may be expensive
            // performance-wise). Also, it should only occur once for a root
            // package. The only drawback is that it may re-activate a root
            // package again, which should only affect performance, but that
            // should be rare. Cycles should still be detected since those
            // will have `DepFeatures` edges.
            RequestedFeatures::CliFeatures(_) => Ok(false),
            RequestedFeatures::DepFeatures {
                features,
                uses_default_features,
            } => {
                let has_default_feature = summary.features().contains_key("default");
                Ok(match self.resolve_features.get(&id) {
                    Some(prev) => {
                        features.is_subset(prev)
                            && (!uses_default_features
                                || prev.contains("default")
                                || !has_default_feature)
                    }
                    None => features.is_empty() && (!uses_default_features || !has_default_feature),
                })
            }
        }
    }

    /// If the package is active returns the `ContextAge` when it was added
    pub fn is_active(&self, id: PackageId) -> Option<ContextAge> {
        self.activations
            .get(&id.as_activations_key())
            .and_then(|(s, l)| if s.package_id() == id { Some(*l) } else { None })
    }

    /// Checks whether all of `parent` and the keys of `conflicting activations`
    /// are still active.
    /// If so returns the `ContextAge` when the newest one was added.
    pub fn is_conflicting(
        &self,
        parent: Option<PackageId>,
        conflicting_activations: &ConflictMap,
    ) -> Option<usize> {
        let mut max = 0;
        if let Some(parent) = parent {
            max = std::cmp::max(max, self.is_active(parent)?);
        }

        for id in conflicting_activations.keys() {
            max = std::cmp::max(max, self.is_active(*id)?);
        }
        Some(max)
    }

    pub fn resolve_replacements(
        &self,
        registry: &RegistryQueryer<'_>,
    ) -> HashMap<PackageId, PackageId> {
        self.activations
            .values()
            .filter_map(|(s, _)| registry.used_replacement_for(s.package_id()))
            .collect()
    }

    pub fn graph(&self) -> Graph<PackageId, std::collections::HashSet<Dependency>> {
        let mut graph: Graph<PackageId, std::collections::HashSet<Dependency>> = Graph::new();
        self.activations
            .values()
            .for_each(|(r, _)| graph.add(r.package_id()));
        for i in self.parents.iter() {
            graph.add(*i);
            for (o, e) in self.parents.edges(i) {
                let old_link = graph.link(*o, *i);
                assert!(old_link.is_empty());
                *old_link = e.iter().cloned().collect();
            }
        }
        graph
    }
}

impl Graph<PackageId, im_rc::HashSet<Dependency>> {
    pub fn parents_of(&self, p: PackageId) -> impl Iterator<Item = (PackageId, bool)> + '_ {
        self.edges(&p)
            .map(|(grand, d)| (*grand, d.iter().any(|x| x.is_public())))
    }
}
