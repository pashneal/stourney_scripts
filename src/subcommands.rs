use crate::config;
use crate::constants;
use crate::dialogue;
use crate::utils;
use log::*;
use splendor_arena::ArenaBuilder;
use std::fs;
use std::path::Path;
use std::time::Duration;
use splendor_arena::tungstenite;

/// Prints the version of the stourney binary
pub fn version_command() {
    println!("stourney v{}", constants::VERSION);
}

/// Guides a user through creating a new project in the specified directory
pub fn new_command(directory: &str) {
    //TODO: Error handling for the expects
    println!("[+] Creating a new project...");
    trace!("[+] Launched the new subcommand");

    if !utils::prereqs_found() {
        println!("[-] Prerequisites not met, exiting...");
        return;
    }

    if !Path::new(&directory).exists() {
        // If the path does not exist, create the empty directory
        fs::create_dir(&directory).expect("[-] Failed to create directory");
    }

    if Path::new(&directory).is_dir() {
        // If the path exists and it is a directory,
        // check if it is empty
        let dir_contents = fs::read_dir(&directory).expect("[-] Failed to read directory contents");
        if dir_contents.count() > 0 {
            if dialogue::confirm_delete() {
                fs::remove_dir_all(&directory).expect("[-] Failed to remove directory");
            } else {
                return;
            }
        }
    } else {
        error!("[-] File exists but is not a directory, cannot overwrite it, exiting...");
        return;
    }

    if utils::create_project(&directory) {
        println!("[+] Project created successfully!");
        config::add_to_recents(&directory);
    } else {
        error!("[-] Failed to create project");
    }
}

/// Guides a user through configuring the stourney binary
pub fn configure_command() {
    let mut num_competitors = dialogue::num_competitors();
    let mut competitors = Vec::new();

    while num_competitors > 0 {
        if let Some(competitor) = dialogue::select_recent_project(competitors.len()) {
            competitors.push(competitor);
            num_competitors -= 1;
        }
    }

    let mut cfg = config::get_config();
    cfg.selected_projects = competitors.clone();
    config::save_config(cfg);

    println!("");
    println!("[+] Configuration saved successfully!");
    config::display_competitors();
    println!("[+] To run the project, try: \n\tstourney run");
}

/// Displays the current competitors in the configuration
pub fn show_competitors() {
    config::display_competitors();
}

/// Sets up the initial arena with configurable settings
fn setup_arena() -> Result<ArenaBuilder, ()> {
    let cfg = config::get_config();
    if cfg.selected_projects.is_empty() {
        println!("No competitors selected yet!");
        println!("try running \n\tstourney config edit\nto add some competitors!");
        return Err(());
    }

    println!("[+] Running the tournament...");
    let mut binaries = Vec::new();
    let port: u16 = 3030;
    let initial_time = Duration::from_secs(10);
    let increment = Duration::from_secs(1);
    let mut interpreter = None;
    let mut static_files = None;

    for competitor in cfg.selected_projects {
        let project_type = utils::guess_project_type(&competitor);
        match project_type {
            utils::ProjectType::Rust | utils::ProjectType::Python => {
                interpreter = Some(utils::python_interpreter_path(&competitor));
            }
            utils::ProjectType::Unknown => {
                error!("[-] Unknown project type for {}", competitor);
                error!("[-] Expected a Rust or Python project");
                println!("[-] Exiting...");
                return Err(());
            }
        }

        match project_type {
            utils::ProjectType::Rust => {
                utils::build_rust_project(&competitor);
                binaries.push(utils::rust_binary_path(&competitor));
            }
            utils::ProjectType::Python => {
                binaries.push(utils::python_binary_path(&competitor));
            }
            _ => {}
        }

        static_files = Some(utils::static_files_path(&competitor));
    }
    info!("Launching the arena...");
    trace!("Port: {}", port);
    trace!("Initial time: {:?}", initial_time);
    trace!("Increment: {:?}", increment);
    trace!("Interpreter: {:?}", interpreter);
    trace!("Static files: {:?}", static_files);
    trace!("Binaries: {:?}", binaries);

    let arena = ArenaBuilder::new()
        .port(port)
        .binaries(binaries)
        .initial_time(initial_time)
        .increment(increment)
        .python_interpreter(&interpreter.unwrap())
        .static_files(&static_files.unwrap());
    Ok(arena)
}
/// Guides a user through running a competition
pub async fn run_command() {
    if let Ok(arena) = setup_arena() {
        let arena = arena.build();
        arena.launch().await;
    }
}

/// Attempts to update the stourney projects that exist in the recents list
pub fn update_command() {
    println!("[+] Updating stourney projects...");
    config::purge_recents();
    let projects = utils::out_of_date_projects();
    for project in projects {
        println!("[+] Updating project: {}...", project);
        utils::update_scaffolding(&project);
    }
}

/// Guides a user through running (and watching) a competition
pub async fn watch_command() {
    if let Ok(arena) = setup_arena() {
        let arena = arena.send_to_web(true, &config::get_config().api_key);
        let arena = arena.build();
        arena.launch().await;
    }
}
