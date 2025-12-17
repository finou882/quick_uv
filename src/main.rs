use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

type AnyResult<T> = Result<T, Box<dyn Error>>;

fn main() -> AnyResult<()> {
    println!("Checking uv installation...");
    if find_uv_executable().is_none() {
        println!("uv not found, installing...");
        install_uv()?;
    }

    let uv_path = find_uv_executable()
        .ok_or("uv installation did not yield a runnable binary")?;

    let project_dir = prompt_for_directory()?;
    ensure_directory(&project_dir)?;

    println!("Running uv init in {}", project_dir.display());
    run_command(&uv_path, &["init"], Some(&project_dir))?;

    println!("Running git init in {}", project_dir.display());
    run_command("git", &["init"], Some(&project_dir))?;

    println!("All done!");
    Ok(())
}

fn prompt_for_directory() -> AnyResult<PathBuf> {
    let mut input = String::new();
    print!("Enter project directory name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Directory name cannot be empty".into());
    }
    Ok(env::current_dir()?.join(trimmed))
}

fn ensure_directory(path: &Path) -> AnyResult<()> {
    if path.exists() {
        if !path.is_dir() {
            return Err(format!("{} exists but is not a directory", path.display()).into());
        }
    } else {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn run_command<P>(program: P, args: &[&str], dir: Option<&Path>) -> AnyResult<()>
where
    P: AsRef<OsStr>,
{
    let program_ref = program.as_ref();
    let display_program = program_ref.to_string_lossy();
    let display_args = args.join(" ");
    println!("> {} {}", display_program, display_args);

    let mut cmd = Command::new(program_ref);
    cmd.args(args);
    if let Some(directory) = dir {
        cmd.current_dir(directory);
    }

    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Command '{}' exited with {}", display_program, status).into())
    }
}

fn install_uv() -> AnyResult<()> {
    if cfg!(target_os = "windows") {
        let script = "Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass; irm https://astral.sh/uv/install.ps1 | iex";
        run_command("powershell.exe", &["-NoProfile", "-Command", script], None)
    } else {
        let script = "curl -LsSf https://astral.sh/uv/install.sh | sh";
        run_command("sh", &["-c", script], None)
    }
}

fn find_uv_executable() -> Option<PathBuf> {
    if let Ok(status) = Command::new("uv").arg("--version").status() {
        if status.success() {
            return Some(PathBuf::from("uv"));
        }
    }

    if cfg!(target_os = "windows") {
        if let Ok(profile) = env::var("USERPROFILE") {
            let candidate = Path::new(&profile).join(".local").join("bin").join("uv.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            let candidate = Path::new(&local_app_data)
                .join("Programs")
                .join("uv")
                .join("uv.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    } else if let Ok(home) = env::var("HOME") {
        let candidate = Path::new(&home).join(".local").join("bin").join("uv");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}
