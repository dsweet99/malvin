use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotfileBackupPayload {
    pub backup_path: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DotfileBackupState {
    Missing,
    Present(DotfileBackupPayload),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DotfileBackupStateRef<'a> {
    Missing,
    Present(&'a DotfileBackupPayload),
}

impl DotfileBackupState {
    #[must_use]
    pub const fn as_slot_state(&self) -> DotfileBackupStateRef<'_> {
        match self {
            Self::Missing => slot_state_ref(None),
            Self::Present(payload) => slot_state_ref(Some(payload)),
        }
    }
}

pub(super) const fn slot_state_ref(payload: Option<&DotfileBackupPayload>) -> DotfileBackupStateRef<'_> {
    match payload {
        None => DotfileBackupStateRef::Missing,
        Some(payload) => DotfileBackupStateRef::Present(payload),
    }
}
