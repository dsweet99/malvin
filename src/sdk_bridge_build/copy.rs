use std::fs;
use std::io;
use std::path::Path;

pub(super) fn copy_dir_recursive(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let dest = to.join(entry.file_name());
        let ftype = entry.file_type()?;
        if ftype.is_dir() {
            copy_dir_recursive(&entry.path(), &dest)?;
        } else if ftype.is_file() {
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}
