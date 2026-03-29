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
