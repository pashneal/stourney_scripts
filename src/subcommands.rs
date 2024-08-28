use crate::utils;
use log::*;
use std::fs;
use std::path::Path;
use crate::dialogue;
use crate::constants;

/// Prints the version of the stourney binary
pub fn version_command() {
    println!("stourney v{}", constants::VERSION);
}

/// Guides a user through creating a new project in the specified directory
pub fn new_command(directory : &str) {
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
    utils::create_project(&directory);
    println!("[+] Project created successfully!");
}

pub fn configure_command(directory : Option<impl AsRef<str>>) {
}
