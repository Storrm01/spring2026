use std::io;
use std::process::Command;

enum FileOperation {
    List(String),           // Directory path
    Display(String),        // File path
    Create(String, String), // File path, content
    Remove(String),         // File path
    Pwd,                    // Print working directory
}

fn perform_operation(operation: FileOperation) {
    match operation {
        FileOperation::List(dir_path) => {
            let result = Command::new("ls").arg(&dir_path).status();

            match result {
                Ok(status) if status.success() => {}
                Ok(_) => eprintln!("Failed to list files in directory: {}", dir_path),
                Err(_) => eprintln!("Failed to execute ls"),
            }
        }

        FileOperation::Display(file_path) => {
            let result = Command::new("cat").arg(&file_path).status();

            match result {
                Ok(status) if status.success() => {}
                Ok(_) => eprintln!("Failed to display file: {}", file_path),
                Err(_) => eprintln!("Failed to execute cat"),
            }
        }

        FileOperation::Create(file_path, content) => {
            let user_command = format!("echo '{}' > {}", content, file_path);

            let result = Command::new("sh")
                .arg("-c")
                .arg(&user_command)
                .status();

            match result {
                Ok(status) if status.success() => {
                    println!("File '{}' created successfully.", file_path);
                }
                Ok(_) => eprintln!("Failed to create file: {}", file_path),
                Err(_) => eprintln!("Failed to execute file creation command"),
            }
        }

        FileOperation::Remove(file_path) => {
            let result = Command::new("rm").arg(&file_path).status();

            match result {
                Ok(status) if status.success() => {
                    println!("File '{}' removed successfully.", file_path);
                }
                Ok(_) => eprintln!("Failed to remove file: {}", file_path),
                Err(_) => eprintln!("Failed to execute rm"),
            }
        }

        FileOperation::Pwd => {
            let result = Command::new("pwd").status();

            match result {
                Ok(status) if status.success() => {}
                Ok(_) => eprintln!("Failed to print working directory"),
                Err(_) => eprintln!("Failed to execute pwd"),
            }
        }
    }
}

fn get_input(prompt: &str) -> String {
    let mut input = String::new();
    println!("{}", prompt);
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

fn main() {
    loop {
        println!("\nWelcome to the File Operations Program!");
        println!("File Operations Menu:");
        println!("1. List files in a directory");
        println!("2. Display file contents");
        println!("3. Create a new file");
        println!("4. Remove a file");
        println!("5. Print working directory");
        println!("0. Exit");

        let choice = get_input("Enter your choice (0-5):");

        match choice.as_str() {
            "1" => {
                let dir_path = get_input("Enter directory path:");
                let operation = FileOperation::List(dir_path);
                perform_operation(operation);
            }

            "2" => {
                let file_path = get_input("Enter file path:");
                let operation = FileOperation::Display(file_path);
                perform_operation(operation);
            }

            "3" => {
                let file_path = get_input("Enter file path:");
                let content = get_input("Enter content:");
                let operation = FileOperation::Create(file_path, content);
                perform_operation(operation);
            }

            "4" => {
                let file_path = get_input("Enter file path:");
                let operation = FileOperation::Remove(file_path);
                perform_operation(operation);
            }

            "5" => {
                let operation = FileOperation::Pwd;
                perform_operation(operation);
            }

            "0" => {
                println!("Goodbye!");
                break;
            }

            _ => {
                println!("Invalid menu option. Please enter a number from 0 to 5.");
            }
        }
    }
}