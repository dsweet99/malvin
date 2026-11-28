use std::ffi::OsString;
use std::path::PathBuf;

use super::options::ModalOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalInvocation {
    pub remote_args: Vec<OsString>,
    pub model: String,
    pub request_files: Vec<PathBuf>,
    pub options: ModalOptions,
}
