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
            is_lossless,
            path
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
        _metadata_count: usize,
        _img: &DynamicImage,
        is_lossless: bool,
        path: &Path
    ) -> u8 {
        // PESO 40%: Punteggio risoluzione (0-40 punti)
        let resolution_score = if megapixels >= 48.0 {
            40  // 48+ MP (8K e oltre)
        } else if megapixels >= 24.0 {
            35  // 24-48 MP (6K)
        } else if megapixels >= 12.0 {
            30  // 12-24 MP (4K)
        } else if megapixels >= 8.0 {
            25  // 8-12 MP (3K)
        } else if megapixels >= 5.0 {
            20  // 5-8 MP (Full HD+)
        } else if megapixels >= 2.0 {
            15  // 2-5 MP (HD)
        } else if megapixels >= 1.0 {
            10  // 1-2 MP
        } else {
            5   // <1 MP
        };
        
        // PESO 60%: Punteggio qualità/compressione (0-60 punti)
        let compression_score = if is_lossless {
            60  // Formato lossless (PNG/TIFF/BMP): massima qualità
        } else {
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase())
                .unwrap_or_default();
                
            if extension == "jpg" || extension == "jpeg" {
                // Calcola bytes per pixel per stimare compressione JPEG
                let total_pixels = megapixels * 1_000_000.0;
                let total_bytes = file_size_mb * 1_024.0 * 1_024.0;
                let bytes_per_pixel = total_bytes / total_pixels;
                
                // Mappa bytes/pixel a punteggio 0-60
                if bytes_per_pixel >= 4.0 {
                    60  // JPEG qualità ~100%
                } else if bytes_per_pixel >= 3.0 {
                    55  // JPEG qualità ~95%
                } else if bytes_per_pixel >= 2.5 {
                    50  // JPEG qualità ~90%
                } else if bytes_per_pixel >= 2.0 {
                    45  // JPEG qualità ~85%
                } else if bytes_per_pixel >= 1.5 {
                    40  // JPEG qualità ~75%
                } else if bytes_per_pixel >= 1.2 {
                    35  // JPEG qualità ~70%
                } else if bytes_per_pixel >= 1.0 {
                    30  // JPEG qualità ~60%
                } else if bytes_per_pixel >= 0.7 {
                    25  // JPEG qualità ~50%
                } else if bytes_per_pixel >= 0.5 {
                    20  // JPEG qualità ~40%
                } else if bytes_per_pixel >= 0.4 {
                    15  // JPEG qualità ~35%
                } else if bytes_per_pixel >= 0.3 {
                    10  // JPEG qualità ~30%
                } else {
                    5   // JPEG qualità <30%
                }
            } else {
                // Altri formati: punteggio medio
                30
            }
        };
        
        // Punteggio totale: 40% risoluzione + 60% qualità/compressione
        (resolution_score + compression_score).min(100)
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
        // Converti da scala 0-100 a 0-5 stelle
        let stars = ((self.quality_score as f32 / 100.0) * 5.0).round() as usize;
        let stars = stars.min(5); // Assicura che non superi 5 stelle
        let filled = "★".repeat(stars);
        let empty = "☆".repeat(5 - stars);
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