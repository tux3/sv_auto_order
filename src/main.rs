#![recursion_limit="256"]

use clap::{App, Arg};
use rayon::prelude::*;
use std::error::Error;
use std::collections::{HashMap, HashSet};

mod file;
use file::File;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = App::new("SV Auto Order")
        .about("Detect compilation order for SystemVerilog files")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Print more details"),
        )
        .arg(
            Arg::with_name("sources")
                .help("The source files")
                .required(true)
                .multiple(true)
        )
        .get_matches();

    let verbose = args.is_present("verbose");
    let filepaths: Vec<_> = args.values_of_os("sources").unwrap().collect();
    let files = filepaths.into_par_iter()
        .inspect(|f| if verbose { println!("Parsing {}", f.to_string_lossy()) } )
        .map(File::new)
        .collect::<Result<Vec<_>, _>>()?;

    let mut module_defs: HashMap<String, &File> = HashMap::new();
    let mut package_defs: HashMap<String, &File> = HashMap::new();

    if verbose {
        println!("Resolving dependencies");
    }
    for file in &files {
        for module_def in &file.modules_defined {
            module_defs.insert(module_def.clone(), file);
        }
        for package_def in &file.packages_defined {
            package_defs.insert(package_def.clone(), file);
        }
    }

    let mut file_users: HashMap<&File, HashSet<&File>> = files.iter().map(|f| (f, HashSet::new())).collect();
    let mut file_deps: HashMap<&File, HashSet<&File>> = HashMap::new();
    for file in &files {
        let mut deps = HashSet::new();
        for module_use in &file.modules_used {
            if let Some(&dep) = module_defs.get(module_use) {
                if dep == file {
                    continue
                }
                file_users.get_mut(dep).unwrap().insert(file);
                if deps.insert(dep) && verbose {
                    println!("{} uses a module from {}", file.name.to_string_lossy(), dep.name.to_string_lossy());
                }
            }
        }
        for package_use in &file.packages_used {
            if let Some(&dep) = package_defs.get(package_use) {
                if dep == file {
                    continue
                }
                file_users.get_mut(dep).unwrap().insert(file);
                if deps.insert(dep) && verbose {
                    println!("{} uses a package/class from {}", file.name.to_string_lossy(), dep.name.to_string_lossy());
                }
            }
        }
        file_deps.insert(file, deps);
    }

    let roots = file_users.into_iter().filter_map(|(f, u)| {
        if u.is_empty() {
            Some(f)
        } else {
            None
        }
    });

    if verbose {
        print!("Ordered source files: ");
    }

    let mut visited_files = HashSet::new();
    for root in roots.into_iter() {
        print_deps_recursive(root, &file_deps, &mut visited_files);
        print!("{} ", root.name.to_string_lossy());
    }

    Ok(())
}

fn print_deps_recursive<'f>(file: &File, file_deps: &HashMap<&File, HashSet<&'f File>>,
                        visited_files: &mut HashSet<&'f File>) {
    let deps = file_deps.get(file).unwrap();
    for &dep in deps {
        if visited_files.insert(dep) {
            print_deps_recursive(dep, file_deps, visited_files);
            print!("{} ", dep.name.to_string_lossy());
        }
    }
}
