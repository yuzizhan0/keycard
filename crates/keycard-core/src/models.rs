//! Plain metadata types (no ciphertext or secret fields).

/// Public metadata for an API key entry — **never** includes nonce, ciphertext, or plaintext secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryMeta {
    pub id: String,
    pub provider: Option<String>,
    pub alias: String,
    pub tags: Option<String>,
    pub created_at: i64,
}

/// A named environment profile (`OPENAI_API_KEY` → entry id mappings are stored separately).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileMeta {
    pub id: String,
    pub name: String,
}
