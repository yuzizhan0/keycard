//! Plain metadata types (no ciphertext or secret fields).

/// Classifies a vault entry for UI and workflows (model/API secrets vs general passwords).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    Api,
    Password,
}

impl EntryKind {
    pub fn as_db_str(self) -> &'static str {
        match self {
            EntryKind::Api => "api",
            EntryKind::Password => "password",
        }
    }

    pub fn from_db(s: Option<&str>) -> Self {
        match s {
            Some("password") => Self::Password,
            _ => Self::Api,
        }
    }
}

/// Public metadata for a vault entry — **never** includes nonce, ciphertext, or plaintext secret.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EntryMeta {
    pub id: String,
    pub provider: Option<String>,
    pub alias: String,
    pub tags: Option<String>,
    pub created_at: i64,
    pub kind: EntryKind,
}

/// A named environment profile (`OPENAI_API_KEY` → entry id mappings are stored separately).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProfileMeta {
    pub id: String,
    pub name: String,
}

/// Saved terminal command (`argv[0]` = program). Optional `profile_id` loads env like `keycard run -p`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CliFavoriteMeta {
    pub id: String,
    pub name: String,
    pub profile_id: Option<String>,
    pub argv: Vec<String>,
    pub notes: Option<String>,
}
