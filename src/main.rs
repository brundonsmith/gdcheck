mod gdproject_metadata;
mod gdscript;
mod godot_project;
mod utils;

use std::{collections::HashMap, convert::TryInto, env::current_dir, path::PathBuf, rc::Rc};

use gdproject_metadata::ast::GDProjectMetadata;
use godot_project::GodotProject;
use utils::slice::Slice;
use walkdir::WalkDir;

use crate::gdscript::{
    ast::ModuleID,
    check::{CheckContext, Checkable},
    parse::parse_script,
};

fn main() -> Result<(), ()> {
    let files = find_files();

    let project_code = std::fs::read_to_string(files.gdproject_metadata.unwrap()).unwrap();
    let metadata: GDProjectMetadata = Slice::new(Rc::new(project_code)).try_into().unwrap();

    let mut scripts = HashMap::new();

    for script in files.gdscripts {
        let module_id = ModuleID(Rc::new(script.to_string_lossy().to_string()));
        let script_code = std::fs::read_to_string(script.clone()).unwrap();
        let parsed = parse_script(module_id.clone(), Slice::new(Rc::new(script_code))).unwrap();

        println!("\n\n\n\n{}\n\n{:?}", script.to_string_lossy(), parsed);
        scripts.insert(module_id, parsed);
    }

    let godot_project = GodotProject {
        metadata,
        rule_severity: HashMap::new(),
        scripts,
    };

    let mut errors = Vec::new();

    for (module_id, script) in godot_project.scripts.iter() {
        println!("Checking {}", module_id.0.as_str());

        script.declarations.check(
            CheckContext {
                module_id,
                godot_project: &godot_project,
            },
            &mut |err| errors.push(err),
        )
    }

    println!("\n\n{:?}", errors);

    Ok(())
}

fn find_files() -> FoundFiles {
    let mut gdproject_metadata = None;
    let mut gdscripts = Vec::new();

    for entry in WalkDir::new(current_dir().unwrap()) {
        let entry = entry.unwrap();

        if entry
            .path()
            .file_name()
            .map(|name| name == "project.godot")
            .unwrap_or(false)
        {
            gdproject_metadata = Some(entry.into_path());
        } else if entry
            .path()
            .extension()
            .map(|ext| ext == "gd")
            .unwrap_or(false)
        {
            gdscripts.push(entry.into_path());
        }
    }

    FoundFiles {
        gdproject_metadata,
        gdscripts,
    }
}

#[derive(Debug, Clone)]
struct FoundFiles {
    pub gdproject_metadata: Option<PathBuf>,
    pub gdscripts: Vec<PathBuf>,
}
