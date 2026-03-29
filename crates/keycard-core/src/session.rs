//! In-memory unlocked data encryption key; zeroized on drop via [`zeroize::Zeroizing`].

use zeroize::Zeroizing;

/// 32-byte DEK derived from the master password; memory is cleared when dropped.
pub type UnlockedDek = Zeroizing<[u8; 32]>;
