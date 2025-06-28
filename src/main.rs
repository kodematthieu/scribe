use std::{fs::File, io, path::PathBuf};

use clap::Parser;
use filetree::FileTree;
use ignore::{WalkBuilder, overrides::OverrideBuilder};

mod filetree;
mod output;

#[derive(Parser)]
struct Command {
    #[arg(default_value = ".")]
    target: PathBuf,

    #[arg(long, short, name = "FILE_NAME")]
    output: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = Command::parse();
    let mut walker = WalkBuilder::new(&args.target);

    if let Some(ref path) = args.output {
        walker.overrides(OverrideBuilder::new(path).build().unwrap());
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
