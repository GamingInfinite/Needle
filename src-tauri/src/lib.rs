use reqwest::{blocking::get, Client};
use std::{
    env::temp_dir,
    fs::{self, File},
    io::{copy, Write},
    path::{Path, PathBuf},
    process::Command,
};
use tokio::io::AsyncWriteExt;
use zip::ZipArchive;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn download_and_extract_bepinex(ss_path: String) -> Result<String, String> {
    let win_url = "https://github.com/BepInEx/BepInEx/releases/download/v5.4.23.3/BepInEx_win_x64_5.4.23.3.zip";

    let mut temp_path: PathBuf = temp_dir();
    temp_path.push("bepinex.zip");

    let res = get(win_url).map_err(|e| format!("Failed to fetch file: {}", e))?;
    let bytes = res
        .bytes()
        .map_err(|e| format!("Failed to read bytes: {}", e))?;

    {
        let mut file =
            File::create(&temp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;
        file.write_all(&bytes)
            .map_err(|e| format!("Failed to write zip: {}", e))?;
    }

    let ss_buf = PathBuf::from(ss_path);
    let file = File::open(&temp_path).map_err(|e| format!("Failed to open temp zip: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to access file in archive: {}", e))?;
        let outpath = ss_buf.join(file.mangled_name());

        if file.name().ends_with("/") {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create dir in extract: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent dir: {}", e))?;
                }
            }
            let mut outfile = File::create(&outpath)
                .map_err(|e| format!("Failed to create extracted file: {}", e))?;
            copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to copy file contents: {}", e))?;
        }
    }

    Ok("Downloaded and Extracted Bepinex".to_string())
}

#[tauri::command]
fn delete_bepinex_files(ss_path: String) -> Result<String, String> {
    let base = Path::new(&ss_path);

    // List of targets (true = directory, false = file)
    let targets: [(&str, bool); 6] = [
        ("BepInEx", true),
        (".doorstop_version", false),
        ("changelog.txt", false),
        ("doorstop_config.ini", false),
        ("winhttp.dll", false),
        ("winhttp.disabled", false),
    ];

    for (name, is_dir) in targets {
        let target_path = base.join(name);
        if target_path.exists() {
            if is_dir {
                fs::remove_dir_all(&target_path)
                    .map_err(|e| format!("Failed to remove directory {:?}: {}", target_path, e))?;
            } else {
                fs::remove_file(&target_path)
                    .map_err(|e| format!("Failed to remove file {:?}: {}", target_path, e))?;
            }
        }
    }

    Ok(format!("Cleaned modding files from {}", ss_path))
}

#[tauri::command]
fn extract_zip(zip_path: String, dest_path: String) -> Result<(), String> {
    let zip_file = File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(zip_file).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = Path::new(&dest_path).join(file.mangled_name());

        if file.name().ends_with('/') {
            // Directory
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            // File
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
            copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    delete_path(&zip_path).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn download_command(url: String, save_path: String) -> Result<(), String> {
    download_file(&url, &save_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_mod(path: String) -> Result<(), String> {
    delete_path(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_game(path: String, args: Vec<String>) {
    let exe_path = Path::new(&path);

    if !exe_path.exists() {
        return;
    }

    let _ = Command::new(exe_path).args(&args).spawn();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            download_and_extract_bepinex,
            delete_bepinex_files,
            download_command,
            delete_mod,
            extract_zip,
            open_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub async fn download_file(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Send request
    let response = Client::new().get(url).send().await?.error_for_status()?; // fail if not 200 OK

    // Get body bytes
    let bytes = response.bytes().await?;

    // Ensure parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Write to file
    let mut file = tokio::fs::File::create(path).await?;
    file.write_all(&bytes).await?;

    Ok(())
}

pub fn delete_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    if path.exists() {
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| e.to_string())?;
        } else {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
