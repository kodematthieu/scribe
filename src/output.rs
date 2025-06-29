use std::{
    fs::File,
    io::{self, BufRead, BufReader, Seek},
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

    match File::open(&full_path) {
        Ok(mut file) => {
            // PASS 1: Validate file content is UTF-8 and count lines.
            // This avoids allocating memory for the whole file.
            let mut line_count: usize = 0;
            let mut is_binary = false;
            {
                let reader = BufReader::new(&mut file);
                for line in reader.lines() {
                    if line.is_err() {
                        is_binary = true;
                        break;
                    }
                    line_count += 1;
                }
            } // `reader` is dropped here, releasing the mutable borrow on `file`.

            if is_binary {
                writeln!(
                    writer,
                    "   * [Could not read file content (likely binary or permission error)]"
                )?;
            } else {
                // PASS 2: Rewind file and print with correct line number padding.
                file.seek(io::SeekFrom::Start(0))?;
                let reader = BufReader::new(file);

                let width = if line_count == 0 {
                    1
                } else {
                    line_count.ilog10() as usize + 1
                };

                // This re-read is safe because we validated the content in pass 1.
                for (i, line_result) in reader.lines().enumerate() {
                    let line = line_result.unwrap();
                    let line_num = i + 1;
                    writeln!(writer, "{: >width$} {}", line_num, line, width = width)?;
                }
            }
        }
        Err(_) => {
            // Catches file open errors (e.g., permissions).
            writeln!(
                writer,
                "   * [Could not read file content (likely binary or permission error)]"
            )?;
        }
    }

    writeln!(writer, "---")?;

    Ok(())
}
