use std::env;
use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::Path;
use std::process::Command;
use uuid::Uuid;
use reqwest;
use std::io;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command_dir = env::var("USERPROFILE").unwrap() + "/commands";
    let log_dir = command_dir.clone() + "/logs";

    fs::create_dir_all(&log_dir).expect("Failed to create logs directory");

    if args.len() < 2 {
        eprintln!("Usage: rust_command_loader <command> [arguments]");
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: rust_command_loader add <command_name> [url_or_path]");
                return;
            }
            let command_name = &args[2];
            let url_or_path = args.get(3);
            add_command(&command_dir, command_name, url_or_path);
        }
        "edit" => {
            if args.len() < 3 {
                eprintln!("Usage: rust_command_loader edit <command_name>");
                return;
            }
            let command_name = &args[2];
            edit_command(&command_dir, command_name);
        }
        "load" | "reload" => {
            if args.len() < 3 {
                eprintln!("Usage: rust_command_loader load/reload <command_name or 'all'>");
                return;
            }
            let target = &args[2];
            if target == "all" {
                reload_all_commands(&command_dir, &log_dir);
            } else {
                compile_command(&command_dir, &log_dir, target);
            }
        }
        "showlogs" => {
            if args.len() < 3 {
                eprintln!("Usage: rust_command_loader showlogs <log_id or 0>");
                return;
            }
            let log_id = &args[2];
            show_logs(&log_dir, log_id);
        }
        "delete" => {
            if args.len() < 3 {
                eprintln!("Usage: rust_command_loader delete <command_name> [-y]");
                return;
            }
            let command_name = &args[2];
            let force = args.get(3).map_or(false, |arg| arg == "-y");
            delete_command(&command_dir, command_name, force);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

fn add_command(command_dir: &str, command_name: &str, url_or_path: Option<&String>) {
    let file_path = format!("{}/{}.rs", command_dir, command_name);

    if let Some(source) = url_or_path {
        if source.starts_with("http://") || source.starts_with("https://") {
            // Download from URL
            match reqwest::blocking::get(source) {
                Ok(response) => {
                    if response.status().is_success() {
                        fs::write(&file_path, response.text().unwrap()).expect("Failed to write command file");
                        println!("Downloaded and created new command file: {}", file_path);
                    } else {
                        eprintln!("Failed to download from URL: {}", source);
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to download from URL: {}", e);
                    return;
                }
            }
        } else {
            // Copy from local path
            if let Err(e) = fs::copy(source, &file_path) {
                eprintln!("Failed to copy from path: {}", e);
                return;
            }
            println!("Copied command file from: {} to {}", source, file_path);
        }
    } else {
        // Create new file with default content
        let default_code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
        fs::create_dir_all(command_dir).expect("Failed to create commands directory");
        fs::write(&file_path, default_code).expect("Failed to write default command file");
        println!("Created new command file: {}", file_path);
    }

    edit_command(command_dir, command_name);
}

fn edit_command(command_dir: &str, command_name: &str) {
    let file_path = format!("{}/{}.rs", command_dir, command_name);
    let editor = env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());
    Command::new(editor).arg(&file_path).status().expect("Failed to open editor");
    compile_command(command_dir, &format!("{}/logs", command_dir), command_name);
}

fn reload_all_commands(command_dir: &str, log_dir: &str) {
    let commands_path = Path::new(command_dir);
    let mut success = true;

    for entry in fs::read_dir(commands_path).expect("Failed to read commands directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let command_name = path.file_stem().and_then(|s| s.to_str()).expect("Invalid file name");
            if !compile_command(command_dir, log_dir, command_name) {
                success = false;
            }
        }
    }

    if success {
        println!("All commands reloaded successfully.");
    } else {
        println!("Some commands failed to reload. Check logs for details.");
    }
}

fn show_logs(log_dir: &str, log_id: &str) {
    if log_id == "0" {
        println!("Log directory: {}", log_dir);
        return;
    }

    let log_path = format!("{}/{}.log", log_dir, log_id);
    if let Ok(contents) = fs::read_to_string(log_path) {
        println!("{}", contents);
    } else {
        println!("Log file not found for ID: {}", log_id);
    }
}

fn delete_command(command_dir: &str, command_name: &str, force: bool) {
    let rs_path = format!("{}/{}.rs", command_dir, command_name);
    let exe_path = format!("{}/{}.exe", command_dir, command_name);

    if !force {
        println!("Are you sure you want to delete the command '{}'? (y/N)", command_name);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        if input.trim().to_lowercase() != "y" {
            println!("Command deletion cancelled.");
            return;
        }
    }

    if fs::remove_file(&rs_path).is_ok() {
        println!("Deleted source file: {}", rs_path);
    }
    if fs::remove_file(&exe_path).is_ok() {
        println!("Deleted executable: {}", exe_path);
    }

    println!("Command '{}' has been deleted.", command_name);
}

fn compile_command(command_dir: &str, log_dir: &str, command_name: &str) -> bool {
    let rs_path = format!("{}/{}.rs", command_dir, command_name);
    let exe_path = format!("{}/{}.exe", command_dir, command_name);
    let log_path = format!("{}/{}.log", log_dir, Uuid::new_v4().to_string());

    let rs_metadata = fs::metadata(&rs_path).expect("Failed to get metadata for source file");
    let exe_metadata = fs::metadata(&exe_path).ok();

    let needs_recompilation = exe_metadata
        .map(|meta| meta.modified().unwrap() < rs_metadata.modified().unwrap())
        .unwrap_or(true);

    if needs_recompilation {
        println!("Compiling command: {}", command_name);
        let output = Command::new("rustc")
            .arg("--out-dir")
            .arg(command_dir)
            .arg(&rs_path)
            .output()
            .expect("Failed to compile command");

        if !output.status.success() {
            eprintln!("Failed to compile command {}. Check logs for details.", command_name);
            let mut log_file = File::create(&log_path).expect("Failed to create log file");
            writeln!(log_file, "Failed to compile command {}: {}", command_name, String::from_utf8_lossy(&output.stderr))
                .expect("Failed to write to log file");
            return false;
        }
    }

    true
}