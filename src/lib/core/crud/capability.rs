use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::{CrudObjectKind, CrudVerb};

/// Capability description for a single object kind.
#[derive(Clone, Debug)]
pub struct CapabilityEntry {
    pub object: CrudObjectKind,
    pub verbs: BTreeSet<CrudVerb>,
    pub notes: Option<String>,
}

impl CapabilityEntry {
    pub fn new(object: CrudObjectKind) -> Self {
        Self {
            object,
            verbs: BTreeSet::new(),
            notes: None,
        }
    }

    pub fn with_verb(mut self, verb: CrudVerb) -> Self {
        self.verbs.insert(verb);
        self
    }

    pub fn allows(&self, verb: CrudVerb) -> bool {
        self.verbs.contains(&verb)
    }

    pub fn verb_names(&self) -> Vec<&'static str> {
        self.verbs.iter().map(|verb| verb.as_str()).collect()
    }
}

/// Mapping of object kinds to their supported verbs.
#[derive(Clone, Debug, Default)]
pub struct CapabilityMap {
    entries: BTreeMap<CrudObjectKind, CapabilityEntry>,
}

impl CapabilityMap {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn upsert_entry(&mut self, entry: CapabilityEntry) {
        self.entries.insert(entry.object.clone(), entry);
    }

    pub fn allow(&mut self, object: CrudObjectKind, verb: CrudVerb) {
        let entry = self
            .entries
            .entry(object.clone())
            .or_insert_with(|| CapabilityEntry::new(object));
        entry.verbs.insert(verb);
    }

    pub fn allows(&self, object: &CrudObjectKind, verb: CrudVerb) -> bool {
        self.entries
            .get(object)
            .map(|entry| entry.allows(verb))
            .unwrap_or(false)
    }

    pub fn entries(&self) -> impl Iterator<Item = &CapabilityEntry> {
        self.entries.values()
    }
}

impl fmt::Display for CapabilityEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} => [{}]",
            self.object.as_str(),
            self.verb_names().join(", ")
        )
    }
}
