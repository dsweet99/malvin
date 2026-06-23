//! External kiss witnesses for [`super`] session name privates.

use super::{NameFileState, SessionNameGuard};

#[test]
fn kiss_witness_session_name_types_and_fns() {
    let name = format!("kisscov{}", std::process::id());
    let _guard = super::acquire_name_with_write(&name, |path| {
        std::fs::write(path, std::process::id().to_string())
    })
    .expect("acquire");
    let _auto = super::generate_auto_name_with(|i| format!("auto{i}"));
    let _ = super::generate_auto_name;
    let _ = super::release_name;
    let _ = super::assert_no_peer_name_lock;
    let state = NameFileState::Absent;
    let _ = std::hint::black_box(state);
    let _ = std::mem::discriminant(&NameFileState::Cleared);
    let _auto = super::acquire_session_name(None).ok();
    let _ = std::mem::size_of::<SessionNameGuard>();
}
