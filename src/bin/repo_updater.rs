use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The name of the parent directory which repositories reside in.
    /// If not provided, the current directory will be used
    name: Option<String>,
}

fn is_git_repo(path: &str) -> bool {
    let git_path = format!("{}/.git", path);
    std::path::Path::new(&git_path).exists()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let num_of_cpus = available_parallelism().unwrap();

    let mut handles = vec![];

    let path = cli.name.unwrap_or_else(|| std::env::current_dir().unwrap().to_str().unwrap().to_string());

    let directories: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    let dirs = Arc::clone(&directories);

    let handle = std::thread::spawn(move || {
        let mut dirs = dirs.lock().unwrap();

        let entries = std::fs::read_dir(path).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path.to_str().unwrap().to_string());
            }
        }
    });

    handles.push(handle);

    for _i in 0..num_of_cpus.get() {
        let directories: Arc<Mutex<Vec<String>>> = Arc::clone(&directories);

        let handle = std::thread::spawn(move || {
            let mut dirs = directories.lock().unwrap();

            if dirs.len() > 0 {
                let dir = dirs.pop().unwrap();

                drop(dirs);

                if is_git_repo(&dir) {
                    println!("Updating: {}", dir);

                    let output = std::process::Command::new("git")
                        .arg("pull")
                        .current_dir(&dir)
                        .output()
                        .expect("failed to execute process");

                    if !output.status.success() {
                        println!("Updating {} Error: {}", dir, String::from_utf8_lossy(&output.stderr));
                    } else {
                        println!("Success: {}", dir);
                    }
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
