use std::{fs::File, io, path::PathBuf};

use clap::Parser;
use filetree::FileTree;
use ignore::WalkBuilder;

mod filetree;
mod output;

#[derive(Parser)]
struct Command {
    #[arg(default_value = ".")]
    target: PathBuf,

    #[arg(long, short, name = "FILE_NAME")]
    output: Option<PathBuf>,
}

fn main() {
    if let Err(e) = run() {
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run() -> io::Result<()> {
    let args = Command::parse();
    let mut walker = WalkBuilder::new(&args.target);

    let output_path_abs = if let Some(p) = &args.output {
        p.canonicalize().ok()
    } else {
        None
    };

    if let Some(output_abs) = output_path_abs {
        walker.filter_entry(move |entry| {
            entry.path().canonicalize().ok() != Some(output_abs.clone())
        });
    }

    let tree = FileTree::new(&args.target, walker.build()).unwrap();

    let mut writer: Box<dyn io::Write> = if let Some(path) = args.output {
        Box::new(File::create(path)?)
    } else {
        Box::new(io::stdout())
    };

    let root_name = args.target.file_name().unwrap_or(args.target.as_os_str());
    tree.display(root_name, &mut writer)?;

    writeln!(writer, "\n---")?;

    output::format(&args.target, tree, &mut writer)?;

    Ok(())
}
