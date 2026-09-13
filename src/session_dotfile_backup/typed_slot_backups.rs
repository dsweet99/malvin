use super::dotfile_backup_state::{
    DotfileBackupPayload, DotfileBackupState, DotfileBackupStateRef, slot_state_ref,
};

macro_rules! typed_slot_backup {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            Missing,
            Present(DotfileBackupPayload),
        }

        impl From<DotfileBackupState> for $name {
            fn from(value: DotfileBackupState) -> Self {
                match value {
                    DotfileBackupState::Missing => Self::Missing,
                    DotfileBackupState::Present(payload) => Self::Present(payload),
                }
            }
        }

        impl From<$name> for DotfileBackupState {
            fn from(value: $name) -> Self {
                match value {
                    $name::Missing => Self::Missing,
                    $name::Present(payload) => Self::Present(payload),
                }
            }
        }

        impl $name {
            #[must_use]
            pub const fn as_slot_state(&self) -> DotfileBackupStateRef<'_> {
                match self {
                    Self::Missing => slot_state_ref(None),
                    Self::Present(payload) => slot_state_ref(Some(payload)),
                }
            }
        }
    };
}

typed_slot_backup!(MalvinChecksBackup);
typed_slot_backup!(MalvinConfigWorkspaceBackup);
