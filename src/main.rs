
mod gdscript;
mod godot_project;
mod utils;

use std::{convert::TryInto, env::current_dir, path::PathBuf};

use godot_project::GodotProject;
use walkdir::WalkDir;

use crate::gdscript::parse::parse_script;

fn main() -> Result<(), ()> {
    let files = find_files();

    let project_code = std::fs::read_to_string(files.godot_project.unwrap()).unwrap();
    let project: GodotProject = project_code.as_str().try_into().unwrap();
    println!("{:?}", project);

    for script in files.gdscripts {
        let script_code = std::fs::read_to_string(script).unwrap();
        let parsed = parse_script(script_code.as_str()).unwrap();

        println!("{:?}", parsed);
    }

    Ok(())
}

fn find_files() -> FoundFiles {
    let mut godot_project = None;
    let mut gdscripts = Vec::new();

    for entry in WalkDir::new(current_dir().unwrap()) {
        let entry = entry.unwrap();

        if entry
            .path()
            .file_name()
            .map(|name| name == "project.godot")
            .unwrap_or(false)
        {
            godot_project = Some(entry.into_path());
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
        godot_project,
        gdscripts,
    }
}

#[derive(Debug, Clone)]
struct FoundFiles {
    pub godot_project: Option<PathBuf>,
    pub gdscripts: Vec<PathBuf>,
}