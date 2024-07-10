use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use uuid::Uuid;

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
        "cnc" => {
            if args.len() < 3 {
                eprintln!("Usage: rust_command_loader cnc <command_name>");
                return;
            }
            let command_name = &args[2];
            let file_path = format!("{}/{}.rs", command_dir, command_name);

            if !Path::new(&file_path).exists() {
                let default_code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
                fs::create_dir_all(&command_dir).expect("Failed to create commands directory");
                fs::write(&file_path, default_code).expect("Failed to write default command file");
                println!("Created new command file: {}", file_path);
            } else {
                println!("Command file already exists: {}", file_path);
            }

            let editor = env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());
            Command::new(editor).arg(&file_path).status().expect("Failed to open editor");

            compile_command(&command_dir, &log_dir, command_name);
        }
        "cload" => {
            let commands_path = Path::new(&command_dir);
            let mut success = true;

            for entry in fs::read_dir(commands_path).expect("Failed to read commands directory") {
                let entry = entry.expect("Failed to read directory entry");
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    let command_name = path.file_stem().and_then(|s| s.to_str()).expect("Invalid file name");
                    if !compile_command(&command_dir, &log_dir, command_name) {
                        success = false;
                    }
                }
            }

            if success {
                println!("All commands loaded successfully.");
            } else {
                println!("Some commands failed to load. Check logs for details.");
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
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
