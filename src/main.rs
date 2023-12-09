use clap::{App, Arg};
use daemonize::Daemonize;
use std::fs;
use std::thread;
use std::time::Duration;
use toml::Value;

fn main() {
    // Parse command line arguments using clap
    let matches = App::new("Directory Remover")
        .version("1.0")
        .author("Your Name")
        .about("Remove specified directories in the background.")
        .arg(Arg::with_name("foreground")
            .short("f")
            .long("foreground")
            .help("Run in the foreground"))
        .get_matches();

    // Check if the program should run in the foreground
    if matches.is_present("foreground") {
        // Run in the foreground
        remove_directories();
    } else {
        // Run as a daemon in the background
        let daemonize = Daemonize::new()
            .pid_file("/tmp/directory_remover.pid")
            .chown_pid_file(true)
            .start();

        match daemonize {
            Ok(_) => remove_directories(),
            Err(e) => eprintln!("Error daemonizing: {}", e),
        }
    }
}

fn remove_directories() {
    // Read configuration from the TOML file
    let config_str = match fs::read_to_string("/etc/antidotrs/config.toml") {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading config file: {}", e);
            return;
        }
    };

    // Parse TOML configuration
    let config: Value = match config_str.parse() {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Error parsing config file: {}", e);
            return;
        }
    };

    // Extract directories from the configuration
    let directories = match config.get("directories") {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|entry| entry.as_str())
            .map(String::from)
            .collect::<Vec<String>>(),
        _ => {
            eprintln!("Invalid or missing 'directories' field in the config file.");
            return;
        }
    };

    // Infinite loop to repeatedly remove directories
    loop {
        for dir in &directories {
            let thread_dir = dir.clone();

            // Spawn a new thread for each directory
            thread::spawn(move || {
                remove_directory(&thread_dir);
            });
        }

        // Sleep for a specified duration before the next iteration
        thread::sleep(Duration::from_secs(1));
    }
}

fn remove_directory(directory: &str) {
    if let Err(e) = fs::remove_dir_all(directory) {
        eprintln!("Error removing {}: {}", directory, e);
    } else {
        println!("Successfully removed {}", directory);
    }
}
