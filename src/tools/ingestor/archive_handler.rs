// src/tools/ingestor/archive_handler.rs
// Archive extraction and temp folder management

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use flate2::read::GzDecoder;
use tar::Archive;
use zip::ZipArchive;

use crate::tools::ingestor::file_collector::collect_all_files_recursive;

/// Get the base temp directory for archive extraction
pub fn get_archive_temp_dir() -> PathBuf {
    std::env::temp_dir().join("robot_brain_extract")
}

/// Create a unique temp directory for archive extraction
pub fn create_archive_temp_dir(archive_name: &str) -> PathBuf {
    let temp_base = get_archive_temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time should always be after UNIX_EPOCH on modern systems")
        .as_secs();
    
    // Sanitize archive name for directory name
    let sanitized = archive_name
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let folder_name = format!("{}_{}", sanitized, timestamp);
    
    let path = temp_base.join(folder_name);
    fs::create_dir_all(&path).ok();
    path
}

/// Track the most recent archive extraction folder
static LAST_ARCHIVE_FOLDER: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Get the most recently created archive temp folder
pub fn get_recent_archive_temp_folder() -> Option<PathBuf> {
    LAST_ARCHIVE_FOLDER.get().cloned()
}

/// Delete empty folders recursively
pub fn delete_empty_folders(dir: &Path) {
    if !dir.exists() {
        return;
    }
    
    // First, recurse into subdirectories
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                delete_empty_folders(&path);
            }
        }
    }
    
    // Then check if this directory is empty and delete it
    if let Ok(mut entries) = fs::read_dir(dir) {
        if entries.next().is_none() {
            let _ = fs::remove_dir(dir);
        }
    }
}

/// Process archive file and extract to temp directory
pub fn process_archive(archive_path: &Path, _temp_dir: &Path) -> Result<Vec<PathBuf>> {
    let extension = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let file_name = archive_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive");
    
    // Create temp dir for this specific archive
    let extract_dir = create_archive_temp_dir(file_name);
    fs::create_dir_all(&extract_dir)?;
    
    // Track this as the most recent archive folder
    let _ = LAST_ARCHIVE_FOLDER.set(extract_dir.clone());
    
    match extension.as_str() {
        "zip" => extract_zip(archive_path, &extract_dir)?,
        "tar" => extract_tar(archive_path, &extract_dir)?,
        "gz" | "tgz" => {
            if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
                extract_tar_gz(archive_path, &extract_dir)?;
            } else {
                extract_gz(archive_path, &extract_dir)?;
            }
        }
        "bz2" => extract_bz2(archive_path, &extract_dir)?,
        "xz" => extract_xz(archive_path, &extract_dir)?,
        "7z" | "rar" => {
            anyhow::bail!("7z/RAR extraction requires external tools. Install 7zip or unrar.")
        }
        _ => anyhow::bail!("Unsupported archive format: {}", extension),
    }
    
    // Collect all extracted files
    let files = collect_all_files_recursive(&extract_dir)?;
    Ok(files)
}

/// Extract ZIP archive
fn extract_zip(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = dest.join(file.mangled_name());
        
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

/// Extract TAR archive
fn extract_tar(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)?;
    let mut archive = Archive::new(file);
    archive.unpack(dest)?;
    Ok(())
}

/// Extract TAR.GZ archive
fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

/// Extract GZ file (single file)
fn extract_gz(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)?;
    let mut decoder = GzDecoder::new(file);
    
    let file_name = archive_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("extracted");
    let out_path = dest.join(file_name);
    
    let mut outfile = fs::File::create(&out_path)?;
    io::copy(&mut decoder, &mut outfile)?;
    Ok(())
}

/// Extract BZ2 file (single file)
fn extract_bz2(archive_path: &Path, dest: &Path) -> Result<()> {
    
    
    let file = fs::File::open(archive_path)?;
    let mut decoder = bzip2::read::BzDecoder::new(file);
    
    let file_name = archive_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("extracted");
    let out_path = dest.join(file_name);
    
    let mut outfile = fs::File::create(&out_path)?;
    io::copy(&mut decoder, &mut outfile)?;
    Ok(())
}

/// Extract XZ file (single file)
fn extract_xz(archive_path: &Path, dest: &Path) -> Result<()> {
    
    
    let file = fs::File::open(archive_path)?;
    let mut decoder = xz2::read::XzDecoder::new(file);
    
    let file_name = archive_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("extracted");
    let out_path = dest.join(file_name);
    
    let mut outfile = fs::File::create(&out_path)?;
    io::copy(&mut decoder, &mut outfile)?;
    Ok(())
}

use std::io;
