use enum_iterator::IntoEnumIterator;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(IntoEnumIterator, Copy, Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum Action {
    #[serde(rename = "*")]
    All = actions::ALL,
    #[serde(rename = "search")]
    Search = actions::SEARCH,
    #[serde(rename = "documents.*")]
    DocumentsAll = actions::DOCUMENTS_ALL,
    #[serde(rename = "documents.add")]
    DocumentsAdd = actions::DOCUMENTS_ADD,
    #[serde(rename = "documents.get")]
    DocumentsGet = actions::DOCUMENTS_GET,
    #[serde(rename = "documents.delete")]
    DocumentsDelete = actions::DOCUMENTS_DELETE,
    #[serde(rename = "indexes.*")]
    IndexesAll = actions::INDEXES_ALL,
    #[serde(rename = "indexes.create")]
    IndexesAdd = actions::INDEXES_CREATE,
    #[serde(rename = "indexes.get")]
    IndexesGet = actions::INDEXES_GET,
    #[serde(rename = "indexes.update")]
    IndexesUpdate = actions::INDEXES_UPDATE,
    #[serde(rename = "indexes.delete")]
    IndexesDelete = actions::INDEXES_DELETE,
    #[serde(rename = "tasks.*")]
    TasksAll = actions::TASKS_ALL,
    #[serde(rename = "tasks.get")]
    TasksGet = actions::TASKS_GET,
    #[serde(rename = "settings.*")]
    SettingsAll = actions::SETTINGS_ALL,
    #[serde(rename = "settings.get")]
    SettingsGet = actions::SETTINGS_GET,
    #[serde(rename = "settings.update")]
    SettingsUpdate = actions::SETTINGS_UPDATE,
    #[serde(rename = "stats.*")]
    StatsAll = actions::STATS_ALL,
    #[serde(rename = "stats.get")]
    StatsGet = actions::STATS_GET,
    #[serde(rename = "dumps.*")]
    DumpsAll = actions::DUMPS_ALL,
    #[serde(rename = "dumps.create")]
    DumpsCreate = actions::DUMPS_CREATE,
    #[serde(rename = "version")]
    Version = actions::VERSION,
    #[serde(rename = "keys.create")]
    KeysAdd = actions::KEYS_CREATE,
    #[serde(rename = "keys.get")]
    KeysGet = actions::KEYS_GET,
    #[serde(rename = "keys.update")]
    KeysUpdate = actions::KEYS_UPDATE,
    #[serde(rename = "keys.delete")]
    KeysDelete = actions::KEYS_DELETE,
}

impl Action {
    pub fn from_repr(repr: u8) -> Option<Self> {
        use actions::*;
        match repr {
            ALL => Some(Self::All),
            SEARCH => Some(Self::Search),
            DOCUMENTS_ALL => Some(Self::DocumentsAll),
            DOCUMENTS_ADD => Some(Self::DocumentsAdd),
            DOCUMENTS_GET => Some(Self::DocumentsGet),
            DOCUMENTS_DELETE => Some(Self::DocumentsDelete),
            INDEXES_ALL => Some(Self::IndexesAll),
            INDEXES_CREATE => Some(Self::IndexesAdd),
            INDEXES_GET => Some(Self::IndexesGet),
            INDEXES_UPDATE => Some(Self::IndexesUpdate),
            INDEXES_DELETE => Some(Self::IndexesDelete),
            TASKS_ALL => Some(Self::TasksAll),
            TASKS_GET => Some(Self::TasksGet),
            SETTINGS_ALL => Some(Self::SettingsAll),
            SETTINGS_GET => Some(Self::SettingsGet),
            SETTINGS_UPDATE => Some(Self::SettingsUpdate),
            STATS_ALL => Some(Self::StatsAll),
            STATS_GET => Some(Self::StatsGet),
            DUMPS_ALL => Some(Self::DumpsAll),
            DUMPS_CREATE => Some(Self::DumpsCreate),
            VERSION => Some(Self::Version),
            KEYS_CREATE => Some(Self::KeysAdd),
            KEYS_GET => Some(Self::KeysGet),
            KEYS_UPDATE => Some(Self::KeysUpdate),
            KEYS_DELETE => Some(Self::KeysDelete),
            _otherwise => None,
        }
    }

    pub fn repr(&self) -> u8 {
        use actions::*;
        match self {
            Self::All => ALL,
            Self::Search => SEARCH,
            Self::DocumentsAll => DOCUMENTS_ALL,
            Self::DocumentsAdd => DOCUMENTS_ADD,
            Self::DocumentsGet => DOCUMENTS_GET,
            Self::DocumentsDelete => DOCUMENTS_DELETE,
            Self::IndexesAll => INDEXES_ALL,
            Self::IndexesAdd => INDEXES_CREATE,
            Self::IndexesGet => INDEXES_GET,
            Self::IndexesUpdate => INDEXES_UPDATE,
            Self::IndexesDelete => INDEXES_DELETE,
            Self::TasksAll => TASKS_ALL,
            Self::TasksGet => TASKS_GET,
            Self::SettingsAll => SETTINGS_ALL,
            Self::SettingsGet => SETTINGS_GET,
            Self::SettingsUpdate => SETTINGS_UPDATE,
            Self::StatsAll => STATS_ALL,
            Self::StatsGet => STATS_GET,
            Self::DumpsAll => DUMPS_ALL,
            Self::DumpsCreate => DUMPS_CREATE,
            Self::Version => VERSION,
            Self::KeysAdd => KEYS_CREATE,
            Self::KeysGet => KEYS_GET,
            Self::KeysUpdate => KEYS_UPDATE,
            Self::KeysDelete => KEYS_DELETE,
        }
    }
}

pub mod actions {
    pub(crate) const ALL: u8 = 0;
    pub const SEARCH: u8 = 1;
    pub const DOCUMENTS_ALL: u8 = 2;
    pub const DOCUMENTS_ADD: u8 = 3;
    pub const DOCUMENTS_GET: u8 = 4;
    pub const DOCUMENTS_DELETE: u8 = 5;
    pub const INDEXES_ALL: u8 = 6;
    pub const INDEXES_CREATE: u8 = 7;
    pub const INDEXES_GET: u8 = 8;
    pub const INDEXES_UPDATE: u8 = 9;
    pub const INDEXES_DELETE: u8 = 10;
    pub const TASKS_ALL: u8 = 11;
    pub const TASKS_GET: u8 = 12;
    pub const SETTINGS_ALL: u8 = 13;
    pub const SETTINGS_GET: u8 = 14;
    pub const SETTINGS_UPDATE: u8 = 15;
    pub const STATS_ALL: u8 = 16;
    pub const STATS_GET: u8 = 17;
    pub const DUMPS_ALL: u8 = 18;
    pub const DUMPS_CREATE: u8 = 19;
    pub const VERSION: u8 = 20;
    pub const KEYS_CREATE: u8 = 21;
    pub const KEYS_GET: u8 = 22;
    pub const KEYS_UPDATE: u8 = 23;
    pub const KEYS_DELETE: u8 = 24;
}