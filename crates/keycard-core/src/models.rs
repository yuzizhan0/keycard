//! Plain metadata types (no ciphertext or secret fields).

/// Public metadata for an API key entry — **never** includes nonce, ciphertext, or plaintext secret.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EntryMeta {
    pub id: String,
    pub provider: Option<String>,
    pub alias: String,
    pub tags: Option<String>,
    pub created_at: i64,
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
