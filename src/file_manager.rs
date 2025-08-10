use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone)]
pub struct FileManager {
    pub folder1: PathBuf,
    pub folder2: PathBuf,
    pub output_folder: PathBuf,
}

impl FileManager {
    pub fn new(folder1: PathBuf, folder2: PathBuf) -> Result<Self> {
        let output_folder = PathBuf::from("output");
        
        if !folder1.exists() {
            anyhow::bail!("Folder 1 does not exist: {:?}", folder1);
        }
        
        if !folder2.exists() {
            anyhow::bail!("Folder 2 does not exist: {:?}", folder2);
        }
        
        fs::create_dir_all(&output_folder)
            .with_context(|| "Failed to create output directory")?;
        
        Ok(FileManager {
            folder1,
            folder2,
            output_folder,
        })
    }
    
    pub fn find_matching_files(&self) -> Result<Vec<(PathBuf, PathBuf)>> {
        let mut folder1_files = HashMap::new();
        let mut matching_pairs = Vec::new();
        
        for entry in WalkDir::new(&self.folder1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_type().is_dir())
        {
            if Self::is_image_file(entry.path()) {
                let file_name = entry.file_name().to_string_lossy().to_string();
                folder1_files.insert(file_name, entry.path().to_path_buf());
            }
        }
        
        for entry in WalkDir::new(&self.folder2)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !e.file_type().is_dir())
        {
            if Self::is_image_file(entry.path()) {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if let Some(path1) = folder1_files.get(&file_name) {
                    matching_pairs.push((path1.clone(), entry.path().to_path_buf()));
                }
            }
        }
        
        matching_pairs.sort_by(|a, b| {
            a.0.file_name().cmp(&b.0.file_name())
        });
        
        Ok(matching_pairs)
    }
    
    fn is_image_file(path: &Path) -> bool {
        match path.extension() {
            Some(ext) => {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                matches!(
                    ext_lower.as_str(),
                    "jpg" | "jpeg" | "png" | "gif" | "bmp" | 
                    "tiff" | "tif" | "webp" | "raw" | "cr2" | 
                    "nef" | "arw" | "dng"
                )
            }
            None => false,
        }
    }
    
    pub fn copy_to_output(&self, source_path: &Path) -> Result<PathBuf> {
        let file_name = source_path
            .file_name()
            .with_context(|| "Failed to get file name")?;
        
        let dest_path = self.output_folder.join(file_name);
        
        if dest_path.exists() {
            let stem = source_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = source_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            
            let mut counter = 1;
            let mut new_dest_path = dest_path.clone();
            while new_dest_path.exists() {
                let new_name = if ext.is_empty() {
                    format!("{}_{}", stem, counter)
                } else {
                    format!("{}_{}.{}", stem, counter, ext)
                };
                new_dest_path = self.output_folder.join(new_name);
                counter += 1;
            }
            
            fs::copy(source_path, &new_dest_path)
                .with_context(|| format!("Failed to copy file to {:?}", new_dest_path))?;
            
            Ok(new_dest_path)
        } else {
            fs::copy(source_path, &dest_path)
                .with_context(|| format!("Failed to copy file to {:?}", dest_path))?;
            
            Ok(dest_path)
        }
    }
    
    pub fn get_relative_path(&self, path: &Path) -> String {
        if path.starts_with(&self.folder1) {
            format!("Folder1/{}", 
                path.strip_prefix(&self.folder1)
                    .unwrap_or(path)
                    .display())
        } else if path.starts_with(&self.folder2) {
            format!("Folder2/{}", 
                path.strip_prefix(&self.folder2)
                    .unwrap_or(path)
                    .display())
        } else {
            path.display().to_string()
        }
    }
}