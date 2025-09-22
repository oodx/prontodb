use std::fmt;
use std::str::FromStr;

/// Logical storage domains recognised by ProntoDB CRUD+.
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CrudDomain {
    /// SQLite-backed storage (primary target for the rebuild).
    Sqlite,
    /// Placeholder for future filesystem domain adapters.
    Filesystem,
}

impl CrudDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            CrudDomain::Sqlite => "sqlite",
            CrudDomain::Filesystem => "filesystem",
        }
    }
}

impl fmt::Display for CrudDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CrudDomain {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "sqlite" => Ok(CrudDomain::Sqlite),
            "filesystem" => Ok(CrudDomain::Filesystem),
            _ => Err("unknown CRUD domain"),
        }
    }
}

/// Object kinds inside a domain (Forge-inspired object taxonomy).
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CrudObjectKind {
    /// SQLite database root (file + connection configuration).
    Base,
    /// Named table within a SQLite database.
    Table,
    /// Row inside a table.
    Record,
    /// Generic container (future expansion: folders, collections).
    Container,
    /// Marker for alias resources.
    Alias,
}

impl CrudObjectKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CrudObjectKind::Base => "base",
            CrudObjectKind::Table => "table",
            CrudObjectKind::Record => "record",
            CrudObjectKind::Container => "container",
            CrudObjectKind::Alias => "alias",
        }
    }
}

impl fmt::Display for CrudObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CrudObjectKind {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "base" => Ok(CrudObjectKind::Base),
            "table" => Ok(CrudObjectKind::Table),
            "record" => Ok(CrudObjectKind::Record),
            "container" => Ok(CrudObjectKind::Container),
            "alias" => Ok(CrudObjectKind::Alias),
            _ => Err("unknown CRUD object kind"),
        }
    }
}

/// CRUD+ verbs (Forge inspired).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CrudVerb {
    Create,
    Read,
    Update,
    Delete,
    List,
    Find,
    Backup,
    Restore,
    Alias,
    Invalid,
}

impl CrudVerb {
    pub fn as_str(&self) -> &'static str {
        match self {
            CrudVerb::Create => "create",
            CrudVerb::Read => "read",
            CrudVerb::Update => "update",
            CrudVerb::Delete => "delete",
            CrudVerb::List => "list",
            CrudVerb::Find => "find",
            CrudVerb::Backup => "backup",
            CrudVerb::Restore => "restore",
            CrudVerb::Alias => "alias",
            CrudVerb::Invalid => "invalid",
        }
    }

    pub const fn all() -> [CrudVerb; 10] {
        [
            CrudVerb::Create,
            CrudVerb::Read,
            CrudVerb::Update,
            CrudVerb::Delete,
            CrudVerb::List,
            CrudVerb::Find,
            CrudVerb::Backup,
            CrudVerb::Restore,
            CrudVerb::Alias,
            CrudVerb::Invalid,
        ]
    }
}

impl fmt::Display for CrudVerb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CrudVerb {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "create" => Ok(CrudVerb::Create),
            "read" => Ok(CrudVerb::Read),
            "update" => Ok(CrudVerb::Update),
            "delete" => Ok(CrudVerb::Delete),
            "list" => Ok(CrudVerb::List),
            "find" => Ok(CrudVerb::Find),
            "backup" => Ok(CrudVerb::Backup),
            "restore" => Ok(CrudVerb::Restore),
            "alias" => Ok(CrudVerb::Alias),
            "invalid" => Ok(CrudVerb::Invalid),
            _ => Err("unknown CRUD verb"),
        }
    }
}
