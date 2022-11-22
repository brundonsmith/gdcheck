mod gdproject_metadata;
mod gdscript;
mod godot_project;
mod utils;

use std::{collections::HashMap, convert::TryInto, env::current_dir, path::PathBuf};

use gdproject_metadata::ast::GDProjectMetadata;
use godot_project::GodotProject;
use walkdir::WalkDir;

use crate::gdscript::{check::Check, parse::parse_script};

fn main() -> Result<(), ()> {
    let files = find_files();

    let project_code = std::fs::read_to_string(files.gdproject_metadata.unwrap()).unwrap();
    let metadata: GDProjectMetadata = (&project_code).try_into().unwrap();

    let mut scripts = HashMap::new();

    for script in files.gdscripts {
        let script_code = std::fs::read_to_string(script.clone()).unwrap();
        let parsed = parse_script(&script_code).unwrap();

        println!("\n\n\n\n{}\n\n{:?}", script.to_string_lossy(), parsed);
        scripts.insert(String::from(script.to_string_lossy()), parsed);
    }

    let project = GodotProject {
        metadata,
        rule_severity: HashMap::new(),
        scripts,
    };

    for (name, script) in project.scripts.iter() {
        println!("Checking {}", name);
        script.check(&project, script, &|_| todo!())
    }

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
