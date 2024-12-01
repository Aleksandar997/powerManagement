use clap::{Parser, Subcommand};
use regex::Regex;
use std::fs::{File};
use std::io::{BufReader, Read, Write};
                              
use std::fs;
use std::io;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    #[clap(visible_alias("l"))]
    #[clap(about("Lists available power modes"))]
    List,
    #[clap(visible_alias("s"))]
    #[clap(about("Sets new power mode"))]
    Set {
        value: String
    },
    #[clap(visible_alias("g"))]
    #[clap(about("Gets current power mode"))]
    GetCurr
}

static POWER_MODES: [&str; 6] = [
    "conservative",
    "ondemand",
    "userspace",
    "powersave",
    "performance",
    "schedutil"
];

static CPU_ROOT_PATH: &str = "/sys/devices/system/cpu";
static SINGLE_CPU_ROOT_FORMAT_PATH: &str = "/cpufreq/scaling_governor";

fn main()-> io::Result<()> {
    let args = Args::parse();

    match args.cmd {
        Commands::List{} => list_command(),
        Commands::Set{value} => set_command(&value)?,
        Commands::GetCurr{} => get_curr_command()?
    }

    Ok(())
}

fn list_command() {
    for power_mode in POWER_MODES {
        println!("{}", power_mode);
    }
}

fn get_curr_command() -> io::Result<()> {
    let cpu_names = get_cpu_names(CPU_ROOT_PATH)?;

    for cpu_name in cpu_names {
        let full_cpu_path = format!("{}{}", CPU_ROOT_PATH, format!("/{}{}", cpu_name, SINGLE_CPU_ROOT_FORMAT_PATH));
        
        let mut file = File::open(full_cpu_path)?; 
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        print!("{}: {}", cpu_name, &contents);
    }

    Ok(())
}

fn set_command(value: &str) -> io::Result<()> {
    if !POWER_MODES.contains(&value) {
        eprintln!("Invalid power mode: {}", value);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid power mode"));
    }

    let cpu_names = match get_cpu_names(CPU_ROOT_PATH) {
        Ok(cpu_names) => cpu_names,
        Err(e) => {
            eprintln!("Error reading CPU names: {}", e);
            return Err(e);
        }
    };

    for cpu_name in cpu_names {
        let full_cpu_path = format!("{}{}{}{}", CPU_ROOT_PATH, "/", cpu_name, SINGLE_CPU_ROOT_FORMAT_PATH);
        
        match write_to_file(&full_cpu_path, value) {
            Ok(()) => println!("Data written successfully to {}", cpu_name),
            Err(e) => eprintln!("Error writing to file: {}", e),
        }
    }

    Ok(())
}

fn write_to_file(filename: &str, data: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

fn get_cpu_names(path: &str) -> io::Result<Vec<String>> {
    let cpu_regex = Regex::new(r"^cpu\d+$").unwrap();

    let files: Vec<String> = fs::read_dir(path)?
        .filter_map(|entry| entry.ok()) 
        .filter(|entry| cpu_regex.is_match(&entry.file_name().to_string_lossy()))
        .map(|entry| entry.file_name().into_string().unwrap_or_default()) 
        .collect();

    return Ok(files);
}