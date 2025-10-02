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

    #[arg(long, short, name = "EXCLUDE_PATTERN")]
    exclude: Vec<String>,

    #[arg(long, short, name = "INCLUDE_PATTERN")]
    include: Vec<String>,

    #[arg(long)]
    rebase: bool,
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
    let mut includes = args.include;

    if args.rebase {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain=v1"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("'git status' failed: {}", stderr),
            ));
        }

        let stdout = String::from_utf8(output.stdout).unwrap();
        for line in stdout.lines() {
            if line.len() < 3 {
                continue;
            }
            let status = &line[0..2];
            if status == "!!" || status == "??" {
                continue;
            }

            if status.as_bytes()[0] != b' ' {
                let path = &line[3..];
                // The path might be a rename `path1 -> path2`. We want the final path.
                let path = path.split(" -> ").last().unwrap();
                // Paths with spaces are quoted.
                let path = path.trim_matches('"');
                includes.push(path.to_string());
            }
        }
    }
    let mut walker = WalkBuilder::new(&args.target);
    walker.add_custom_ignore_filename(".scribeignore");

    let mut override_builder = OverrideBuilder::new(&args.target);

    if !includes.is_empty() {
        walker.add_custom_ignore_filename(".gitignore");
        walker.ignore(false);
        walker.require_git(false);

        for pattern in &includes {
            override_builder.add(pattern).unwrap();
        }
    }

    for pattern in &args.exclude {
        let mut glob = String::from("!");
        glob.push_str(pattern);
        override_builder.add(&glob).unwrap();
    }

    let overrides = override_builder.build().unwrap();
    walker.overrides(overrides);

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
