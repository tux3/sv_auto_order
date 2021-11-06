use std::ffi::{OsStr, OsString};
use std::error::Error;
use sv_parser::{parse_sv, unwrap_node, SyntaxTree, Defines, RefNode};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::hash::{Hash, Hasher};

pub struct File {
    pub name: OsString,
    pub modules_defined: HashSet<String>,
    pub modules_used: HashSet<String>,
    pub packages_defined: HashSet<String>,
    pub packages_used: HashSet<String>,
    pub defines: Defines,
    pub ast: SyntaxTree,
}

impl File {
    pub fn new(path: &OsStr, incdirs: &[&Path]) -> Result<File, Box<dyn Error + Send + Sync>> {
        let defines = HashMap::new();
        let mut incdirs = incdirs.to_vec();
        let parent_dir = Path::new(path).parent().unwrap();
        incdirs.push(parent_dir);
        let (ast, defines) = parse_sv(path, &defines, &incdirs, false, true)?;

        let (modules_defined, modules_used) = Self::collect_modules(&ast);
        let (packages_defined, packages_used) = Self::collect_packages(&ast);

        Ok(File {
            name: path.to_owned(),
            modules_defined,
            modules_used,
            packages_defined,
            packages_used,
            defines,
            ast
        })
    }

    fn collect_modules(ast: &SyntaxTree) -> (HashSet<String>, HashSet<String>) {
        let mut modules_defined = HashSet::new();
        let mut modules_used = HashSet::new();

        for node in ast {
            match node {
                RefNode::ModuleInstantiation(x) => {
                    let id = unwrap_node!(x, ModuleIdentifier).unwrap();
                    let id_str = get_ident_string(ast, id).unwrap();
                    //println!("module used: {}", id_str);
                    modules_used.insert(id_str);
                }
                RefNode::ModuleDeclaration(x) => {
                    let id = unwrap_node!(x, ModuleIdentifier).unwrap();
                    let id_str = get_ident_string(ast, id).unwrap();
                    //println!("module decl: {}", id_str);
                    modules_defined.insert(id_str);
                }
                _ => (),
            }
        }

        (modules_defined, modules_used)
    }

    fn collect_packages(ast: &SyntaxTree) -> (HashSet<String>, HashSet<String>) {
        let mut packages_defined = HashSet::new();
        let mut packages_used = HashSet::new();

        for node in ast {
            match node {
                RefNode::PackageDeclaration(x) => {
                    let id = unwrap_node!(x, PackageIdentifier).unwrap();
                    let id_str = get_ident_string(ast, id).unwrap();
                    //println!("package decl: {}", id_str);
                    packages_defined.insert(id_str);
                }
                RefNode::ClassDeclaration(x) => {
                    let id = unwrap_node!(x, ClassIdentifier).unwrap();
                    let id_str = get_ident_string(ast, id).unwrap();
                    //println!("class decl: {}", id_str);
                    packages_defined.insert(id_str);
                }
                RefNode::PackageImportItem(x) => {
                    let id = unwrap_node!(x, PackageIdentifier).unwrap();
                    let id_str = get_ident_string(ast, id).unwrap();
                    //println!("package used: {}", id_str);
                    packages_used.insert(id_str);
                }
                RefNode::ClassScope(x) => {
                    let id = unwrap_node!(x, ClassIdentifier).unwrap();
                    let id_str = get_ident_string(ast, id).unwrap();
                    //println!("class/package used: {}", id_str);
                    packages_used.insert(id_str);
                }
                _ => (),
            }
        }

        (packages_defined, packages_used)
    }
}

impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for File {}

fn get_ident_string(ast: &SyntaxTree, node: RefNode) -> Option<String> {
    // unwrap_node! can take multiple types
    match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
        Some(RefNode::SimpleIdentifier(x)) => {
            ast.get_str(&x.nodes.0).map(|f| f.to_owned())
        }
        Some(RefNode::EscapedIdentifier(x)) => {
            ast.get_str(&x.nodes.0).map(|f| f.to_owned())
        }
        _ => None,
    }
}
