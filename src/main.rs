use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

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
            let file_path = format!("C:/Users/ZERO/commands/{}.rs", command_name);

            if !Path::new(&file_path).exists() {
                let default_code = r#"
fn main() {
    println!("Hello, world!");
}
"#;
                fs::create_dir_all("C:/Users/ZERO/commands").expect("Failed to create commands directory");
                fs::write(&file_path, default_code).expect("Failed to write default command file");
                println!("Created new command file: {}", file_path);
            } else {
                println!("Command file already exists: {}", file_path);
            }

            let editor = env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());
            Command::new(editor).arg(&file_path).status().expect("Failed to open editor");
        }
        "cload" => {
            let commands_path = "C:/Users/ZERO/commands";
            let mut success = true;

            for entry in fs::read_dir(commands_path).expect("Failed to read commands directory") {
                let entry = entry.expect("Failed to read directory entry");
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    let command_name = path.file_stem().and_then(|s| s.to_str()).expect("Invalid file name");
                    let output = Command::new("rustc")
                        .arg("--out-dir")
                        .arg(commands_path)
                        .arg(&path)
                        .output()
                        .expect("Failed to compile command");

                    if !output.status.success() {
                        eprintln!("Failed to compile command {}: {}", command_name, String::from_utf8_lossy(&output.stderr));
                        success = false;
                    } else {
                        println!("Successfully compiled command: {}", command_name);
                    }
                }
            }

            if success {
                println!("All commands loaded successfully.");
            } else {
                println!("Some commands failed to load.");
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
