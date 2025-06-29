use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use crate::filetree::FileTree;

pub fn format(root: &Path, tree: FileTree, mut out: impl io::Write) -> io::Result<()> {
    tree.visit_files(|_, path| format_file(root, path, &mut out), ())
}

fn format_file(root: &Path, relative_path: &Path, writer: &mut impl io::Write) -> io::Result<()> {
    let full_path = root.join(relative_path);

    writeln!(writer)?;
    writeln!(
        writer,
        "/{}:",
        relative_path.to_string_lossy().replace('\\', "/")
    )?;

    match File::open(full_path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let lines: Vec<String> = match reader.lines().collect() {
                Ok(lines) => lines,
                Err(e) => return Err(e), // Propagate I/O errors during read.
            };

            let total_lines = lines.len();
            let width = if total_lines == 0 {
                1
            } else {
                total_lines.ilog10() as usize + 1
            };

            for (i, line) in lines.iter().enumerate() {
                let line_num = i + 1;
                // Use the calculated `width` to format the line number.
                // The `width = width` syntax passes the variable to the formatter.
                writeln!(writer, "{: >width$} {}", line_num, line)?;
            }
        }
        Err(_) => {
            writeln!(
                writer,
                "   * [Could not read file content (likely binary or permission error)]"
            )?;
        }
    }

    writeln!(writer, "---")?;

    Ok(())
}
