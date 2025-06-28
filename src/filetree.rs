use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    io,
    path::{Path, PathBuf},
};

pub enum FileTree {
    Node(BTreeMap<OsString, FileTree>),
    Leaf,
}
impl FileTree {
    pub fn new(root: &Path, walker: ignore::Walk) -> Result<Self, ignore::Error> {
        let root_name = root.file_name().unwrap_or(root.as_os_str()).to_os_string();
        let mut root_children = BTreeMap::new();

        // The first entry is the root itself; skip it.
        for result in walker.skip(1) {
            let entry = result?;
            let path = entry.path().strip_prefix(root).unwrap();

            let mut current_level = &mut root_children;

            let components: Vec<_> = path.components().collect();
            let num_components = components.len();

            for (i, component) in components.iter().enumerate() {
                let component_name = component.as_os_str().to_os_string();
                let is_last_component = i == num_components - 1;

                if is_last_component {
                    if entry.file_type().unwrap().is_dir() {
                        current_level.insert(component_name, Self::Node(BTreeMap::new()));
                    } else {
                        current_level.insert(component_name, Self::Leaf);
                    }
                } else {
                    let child_node = current_level
                        .entry(component_name)
                        .or_insert_with(|| Self::Node(BTreeMap::new()));

                    if let Self::Node(children) = child_node {
                        current_level = children;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(FileTree::Node(root_children))
    }
    pub fn file_count(&self) -> usize {
        match self {
            Self::Node(children) => children.values().map(|x| x.file_count()).sum(),
            Self::Leaf => 1,
        }
    }
    pub fn visit_files<F, T>(&self, mut f: F, mut ctx: T)
    where
        F: FnMut(&mut T, &Path),
    {
        self.visit_files_recursive(&mut f, &mut ctx, PathBuf::new());
    }

    fn visit_files_recursive<F, T>(&self, f: &mut F, ctx: &mut T, current_path: PathBuf)
    where
        F: FnMut(&mut T, &Path),
    {
        match self {
            Self::Leaf => {
                f(ctx, &current_path);
            }
            Self::Node(children) => {
                for (name, node) in children {
                    let new_path = current_path.join(name);
                    node.visit_files_recursive(f, ctx, new_path);
                }
            }
        }
    }
    pub fn display<W: io::Write>(&self, root_name: &OsStr, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Node(children) => {
                writeln!(writer, "{}", root_name.to_string_lossy())?;
                Self::display_recursive(children, writer, "")?;
            }
            Self::Leaf => {
                writeln!(writer, "{}", root_name.to_string_lossy())?;
            }
        }
        Ok(())
    }

    fn display_recursive<W: io::Write>(
        children: &BTreeMap<OsString, FileTree>,
        writer: &mut W,
        prefix: &str,
    ) -> io::Result<()> {
        let mut iter = children.iter().peekable();

        while let Some((name, node)) = iter.next() {
            let is_last = iter.peek().is_none();
            let connector = if is_last { "└── " } else { "├── " };
            let child_prefix = if is_last { "    " } else { "│   " };

            match node {
                Self::Leaf => {
                    writeln!(writer, "{}{}{}", prefix, connector, name.to_string_lossy())?;
                }

                Self::Node(current_children) => {
                    let mut path_to_print = PathBuf::from(name);

                    let mut node_in_chain = node;
                    let mut children_in_chain = current_children;

                    while children_in_chain.len() == 1 {
                        if let Some((single_child_name, single_child_node)) =
                            children_in_chain.iter().next()
                        {
                            if let FileTree::Node(grandchildren) = single_child_node {
                                // It's a compressible chain. Append the name.
                                path_to_print.push(single_child_name);

                                // Move our pointers down to the next link in the chain.
                                node_in_chain = single_child_node;
                                children_in_chain = grandchildren;
                            } else {
                                // The single child is a file, so the chain ends.
                                break;
                            }
                        }
                    }

                    if let FileTree::Node(final_children) = node_in_chain {
                        if final_children.is_empty() {
                            writeln!(
                                writer,
                                "{}{}{}/",
                                prefix,
                                connector,
                                path_to_print.to_string_lossy()
                            )?;
                        } else if final_children.len() == 1 {
                            let (file_name, _) = final_children.iter().next().unwrap();
                            path_to_print.push(file_name);
                            writeln!(
                                writer,
                                "{}{}{}",
                                prefix,
                                connector,
                                path_to_print.to_string_lossy()
                            )?;
                        } else {
                            writeln!(
                                writer,
                                "{}{}{}/",
                                prefix,
                                connector,
                                path_to_print.to_string_lossy()
                            )?;
                            let new_prefix = format!("{}{}", prefix, child_prefix);
                            Self::display_recursive(final_children, writer, &new_prefix)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
