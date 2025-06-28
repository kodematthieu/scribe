use std::{io, path::Path};

use crate::filetree::FileTree;

const SEPARATOR: &str = "---";

pub fn format(root: &Path, tree: FileTree, mut out: impl io::Write) -> io::Result<()> {
    tree.visit_files(|_, path| format_file(root, path, &mut out), ())
}

fn format_file(root: &Path, relative_path: &Path, writer: &mut impl io::Write) -> io::Result<()> {
    let full_path = root.join(relative_path);

    writeln!(
        writer,
        "{} /{}",
        SEPARATOR,
        relative_path.to_string_lossy().replace('\\', "/")
    )?;

    if let Ok(content) = std::fs::read_to_string(full_path) {
        writeln!(writer, "{}", content)?;
    } else {
        writeln!(
            writer,
            "[Could not read file content (likely binary or permission error)]"
        )?;
    }

    writeln!(writer, "{}", SEPARATOR)?;
    writeln!(writer)?;

    Ok(())
}
