use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use sha2::{Sha256, Digest};
use std::io::Read;

#[derive(Debug, Clone)]
pub struct ImageAnalysis {
    pub file_path: String,
    pub file_size_mb: f64,
    pub width: u32,
    pub height: u32,
    pub megapixels: f64,
    pub metadata_count: usize,
    pub exif_data: Vec<(String, String)>,
    pub quality_score: u8,
    pub hash: String,
}

impl ImageAnalysis {
    pub fn analyze_image(path: &Path) -> Result<Self> {
        let file_path = path.to_string_lossy().to_string();
        
        let metadata = std::fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for {:?}", path))?;
        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
        
        let img = image::open(path)
            .with_context(|| format!("Failed to open image {:?}", path))?;
        let (width, height) = img.dimensions();
        let megapixels = (width as f64 * height as f64) / 1_000_000.0;
        
        let (exif_data, metadata_count) = Self::extract_exif_data(path);
        
        let is_lossless = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "png" | "tiff" | "tif" | "bmp"))
            .unwrap_or(false);
            
        let quality_score = Self::calculate_quality_score(
            file_size_mb,
            megapixels,
            metadata_count,
            &img,
            is_lossless
        );
        
        let hash = Self::calculate_file_hash(path)?;
        
        Ok(ImageAnalysis {
            file_path,
            file_size_mb,
            width,
            height,
            megapixels,
            metadata_count,
            exif_data,
            quality_score,
            hash,
        })
    }
    
    fn extract_exif_data(path: &Path) -> (Vec<(String, String)>, usize) {
        let mut exif_data = Vec::new();
        let mut count = 0;
        
        if let Ok(file) = File::open(path) {
            let mut bufreader = BufReader::new(file);
            let exif_reader = exif::Reader::new();
            if let Ok(exif) = exif_reader.read_from_container(&mut bufreader) {
                for field in exif.fields() {
                    count += 1;
                    let tag_name = format!("{:?}", field.tag);
                    let value = field.display_value().to_string();
                    exif_data.push((tag_name, value));
                }
            }
        }
        
        (exif_data, count)
    }
    
    fn calculate_quality_score(
        file_size_mb: f64,
        megapixels: f64,
        metadata_count: usize,
        _img: &DynamicImage,
        is_lossless: bool
    ) -> u8 {
        let mut score = 0u8;
        
        // Risoluzione (peso maggiore per PNG)
        if megapixels >= 24.0 {
            score += 2;
        } else if megapixels >= 12.0 {
            score += 2;
        } else if megapixels >= 6.0 {
            score += 1;
        } else if megapixels >= 2.0 {
            score += 1;
        }
        
        // Dimensione file (importante per qualità)
        if is_lossless {
            // Per PNG, file più grandi indicano maggior informazione
            if file_size_mb >= 10.0 {
                score += 2;
            } else if file_size_mb >= 5.0 {
                score += 1;
            } else if file_size_mb >= 1.0 {
                score += 1;
            }
        } else {
            // Per JPEG, bilanciare dimensione
            if file_size_mb >= 4.0 {
                score += 1;
            } else if file_size_mb >= 2.0 {
                score += 1;
            }
        }
        
        // Bonus per formato senza perdita
        if is_lossless {
            score += 1;
        }
        
        // Metadata (peso ridotto)
        if metadata_count >= 15 {
            score += 1;
        } else if metadata_count >= 5 {
            score += 1;
        }
        
        score.min(5)
    }
    
    fn calculate_file_hash(path: &Path) -> Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    pub fn get_quality_stars(&self) -> String {
        let filled = "★".repeat(self.quality_score as usize);
        let empty = "☆".repeat(5 - self.quality_score as usize);
        format!("{}{}", filled, empty)
    }
    
    pub fn get_important_metadata(&self) -> Vec<String> {
        let mut result = Vec::new();
        
        for (key, value) in &self.exif_data {
            if key.contains("DateTime") || 
               key.contains("Make") || 
               key.contains("Model") ||
               key.contains("ISO") ||
               key.contains("FNumber") ||
               key.contains("ExposureTime") {
                result.push(format!("{}: {}", key, value));
            }
        }
        
        result
    }
}