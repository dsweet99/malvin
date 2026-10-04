use std::marker::PhantomData;
use std::path::Path;

use super::dotfile_backup_state::{
    DotfileBackupPayload, DotfileBackupState, DotfileBackupStateRef, slot_state_ref,
};
use super::slots::{MALVIN_CHECKS_SLOT, MALVIN_CONFIG_WORKSPACE_SLOT, backup_slot, restore_slot};

pub trait SlotKind {
    const SLOT: usize;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotBackup<K> {
    Missing,
    Present(DotfileBackupPayload, PhantomData<K>),
}

impl<K> From<DotfileBackupState> for SlotBackup<K> {
    fn from(value: DotfileBackupState) -> Self {
        match value {
            DotfileBackupState::Missing => Self::Missing,
            DotfileBackupState::Present(payload) => Self::Present(payload, PhantomData),
        }
    }
}

impl<K> From<SlotBackup<K>> for DotfileBackupState {
    fn from(value: SlotBackup<K>) -> Self {
        match value {
            SlotBackup::Missing => Self::Missing,
            SlotBackup::Present(payload, _) => Self::Present(payload),
        }
    }
}

impl<K: SlotKind> SlotBackup<K> {
    #[must_use]
    pub const fn present(payload: DotfileBackupPayload) -> Self {
        Self::Present(payload, PhantomData)
    }

    #[must_use]
    pub const fn as_slot_state(&self) -> DotfileBackupStateRef<'_> {
        match self {
            Self::Missing => slot_state_ref(None),
            Self::Present(payload, _) => slot_state_ref(Some(payload)),
        }
    }

    pub(super) fn backup(
        work_dir: &Path,
        generate_id: &mut impl FnMut(usize) -> String,
    ) -> Result<Self, String> {
        backup_slot(K::SLOT, work_dir, generate_id).map(Into::into)
    }

    pub(super) fn restore(&self, work_dir: &Path) -> Result<(), String> {
        restore_slot(work_dir, self.as_slot_state(), K::SLOT)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MalvinChecksSlot;

impl SlotKind for MalvinChecksSlot {
    const SLOT: usize = MALVIN_CHECKS_SLOT;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MalvinConfigWorkspaceSlot;

impl SlotKind for MalvinConfigWorkspaceSlot {
    const SLOT: usize = MALVIN_CONFIG_WORKSPACE_SLOT;
}

pub type MalvinChecksBackup = SlotBackup<MalvinChecksSlot>;
pub type MalvinConfigWorkspaceBackup = SlotBackup<MalvinConfigWorkspaceSlot>;
