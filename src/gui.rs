use crate::image_analyzer::ImageAnalysis;
use anyhow::Result;
use eframe::egui;
use egui::{Align, Color32, ColorImage, Context, TextureHandle, Vec2};
use image::{DynamicImage, GenericImageView, imageops::FilterType};
use std::path::Path;

const MAX_TEXTURE_SIZE: u32 = 2048;

pub struct ImageComparisonApp {
    image1_analysis: ImageAnalysis,
    image2_analysis: ImageAnalysis,
    texture1: Option<TextureHandle>,
    texture2: Option<TextureHandle>,
    user_choice: Option<u8>,
    skip: bool,
    exit_program: bool,
    is_loading: bool,
}

impl ImageComparisonApp {
    pub fn new(
        image1_analysis: ImageAnalysis,
        image2_analysis: ImageAnalysis,
    ) -> Self {
        ImageComparisonApp {
            image1_analysis,
            image2_analysis,
            texture1: None,
            texture2: None,
            user_choice: None,
            skip: false,
            exit_program: false,
            is_loading: false,
        }
    }
    
    pub fn run(mut self) -> Result<(Option<u8>, bool)> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1400.0, 900.0])
                .with_title("PhotoScope - Image Comparison"),
            ..Default::default()
        };
        
        let result = std::sync::Arc::new(std::sync::Mutex::new((None, false)));
        let result_clone = result.clone();
        
        eframe::run_simple_native("PhotoScope", options, move |ctx, _frame| {
            self.update(ctx);
            
            if self.user_choice.is_some() || self.skip || self.exit_program {
                *result_clone.lock().unwrap() = (self.user_choice, self.exit_program);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }).map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;
        
        Ok(result.lock().unwrap().clone())
    }
    
    fn update(&mut self, ctx: &Context) {
        if self.texture1.is_none() {
            self.texture1 = self.load_texture(ctx, &self.image1_analysis.file_path, "img1");
        }
        if self.texture2.is_none() {
            self.texture2 = self.load_texture(ctx, &self.image2_analysis.file_path, "img2");
        }
        
        // Panel superiore per l'header con le informazioni
        egui::TopBottomPanel::top("header_panel").show(ctx, |ui| {
            self.show_header(ui);
            ui.separator();
        });
        
        // Panel inferiore per i controlli
        egui::TopBottomPanel::bottom("controls_panel").show(ctx, |ui| {
            ui.separator();
            self.show_controls(ui);
        });
        
        // Panel centrale per le immagini (usa tutto lo spazio rimanente)
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_images(ui);
        });
        
        self.handle_keyboard_input(ctx);
    }
    
    fn show_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let col_width = ui.available_width() / 2.0 - 10.0;
            
            ui.vertical(|ui| {
                ui.set_min_width(col_width);
                ui.set_max_width(col_width);
                
                ui.colored_label(Color32::from_rgb(100, 200, 255), 
                    format!("[1] {}", Path::new(&self.image1_analysis.file_path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()));
                ui.label(format!("Dimensione: {:.2} MB", self.image1_analysis.file_size_mb));
                ui.label(format!("Risoluzione: {:.1} MP ({}x{})", 
                    self.image1_analysis.megapixels,
                    self.image1_analysis.width,
                    self.image1_analysis.height));
                ui.label(format!("Metadata: {} campi", self.image1_analysis.metadata_count));
                ui.label(format!("Qualità: {}", self.image1_analysis.get_quality_stars()));
            });
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.set_min_width(col_width);
                ui.set_max_width(col_width);
                
                ui.colored_label(Color32::from_rgb(255, 200, 100),
                    format!("[2] {}", Path::new(&self.image2_analysis.file_path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()));
                ui.label(format!("Dimensione: {:.2} MB", self.image2_analysis.file_size_mb));
                ui.label(format!("Risoluzione: {:.1} MP ({}x{})", 
                    self.image2_analysis.megapixels,
                    self.image2_analysis.width,
                    self.image2_analysis.height));
                ui.label(format!("Metadata: {} campi", self.image2_analysis.metadata_count));
                ui.label(format!("Qualità: {}", self.image2_analysis.get_quality_stars()));
            });
        });
    }
    
    fn show_images(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let available_height = ui.available_height();
        
        // Debug: stampa le dimensioni reali disponibili
        eprintln!("Panel centrale - Width: {}, Height: {}", available_width, available_height);
        
        // Calcola dimensioni per ogni immagine (metà larghezza, tutta l'altezza)
        let image_max_width = (available_width / 2.0) - 20.0;
        let image_max_height = available_height - 20.0;
        
        ui.horizontal_centered(|ui| {
            // Immagine 1
            if let Some(texture) = &self.texture1 {
                let size = texture.size_vec2();
                let scale = (image_max_width / size.x).min(image_max_height / size.y);
                let scaled_size = Vec2::new(size.x * scale, size.y * scale);
                
                eprintln!("Img1 - Texture: {}x{}, Max area: {}x{}, Scale: {}, Final: {}x{}", 
                    size.x, size.y, image_max_width, image_max_height, scale, scaled_size.x, scaled_size.y);
                
                ui.add_space((image_max_width - scaled_size.x) / 2.0);
                ui.image((texture.id(), scaled_size));
                ui.add_space((image_max_width - scaled_size.x) / 2.0);
            } else {
                ui.add_space(image_max_width / 2.0);
                ui.label("Caricamento immagine 1...");
                ui.add_space(image_max_width / 2.0);
            }
            
            ui.separator();
            
            // Immagine 2
            if let Some(texture) = &self.texture2 {
                let size = texture.size_vec2();
                let scale = (image_max_width / size.x).min(image_max_height / size.y);
                let scaled_size = Vec2::new(size.x * scale, size.y * scale);
                
                eprintln!("Img2 - Texture: {}x{}, Max area: {}x{}, Scale: {}, Final: {}x{}", 
                    size.x, size.y, image_max_width, image_max_height, scale, scaled_size.x, scaled_size.y);
                
                ui.add_space((image_max_width - scaled_size.x) / 2.0);
                ui.image((texture.id(), scaled_size));
                ui.add_space((image_max_width - scaled_size.x) / 2.0);
            } else {
                ui.add_space(image_max_width / 2.0);
                ui.label("Caricamento immagine 2...");
                ui.add_space(image_max_width / 2.0);
            }
        });
    }
    
    fn show_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Premi ");
            ui.colored_label(Color32::from_rgb(100, 200, 255), "[1]");
            ui.label(" o ");
            ui.colored_label(Color32::from_rgb(255, 200, 100), "[2]");
            ui.label(" per scegliere, ");
            ui.colored_label(Color32::from_rgb(200, 200, 200), "[S]");
            ui.label(" per saltare, ");
            ui.colored_label(Color32::from_rgb(255, 100, 100), "[ESC]");
            ui.label(" per uscire");
        });
        
        ui.horizontal(|ui| {
            if ui.button("Scegli Immagine 1").clicked() {
                self.user_choice = Some(1);
            }
            if ui.button("Scegli Immagine 2").clicked() {
                self.user_choice = Some(2);
            }
            if ui.button("Salta").clicked() {
                self.skip = true;
            }
            
            ui.separator();
            
            if ui.button(
                egui::RichText::new("Esci dal Programma")
                    .color(Color32::from_rgb(255, 100, 100))
            ).clicked() {
                self.exit_program = true;
            }
        });
    }
    
    fn handle_keyboard_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
            self.user_choice = Some(1);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
            self.user_choice = Some(2);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.skip = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.exit_program = true;
        }
    }
    
    fn load_texture(&self, ctx: &Context, path: &str, name: &str) -> Option<TextureHandle> {
        match image::open(path) {
            Ok(mut img) => {
                // Ridimensiona l'immagine se è troppo grande per evitare problemi di memoria
                let (width, height) = img.dimensions();
                if width > MAX_TEXTURE_SIZE || height > MAX_TEXTURE_SIZE {
                    let ratio = (MAX_TEXTURE_SIZE as f32 / width.max(height) as f32).min(1.0);
                    let new_width = (width as f32 * ratio) as u32;
                    let new_height = (height as f32 * ratio) as u32;
                    img = img.resize(new_width, new_height, FilterType::Lanczos3);
                }
                
                let color_image = self.dynamic_image_to_color_image(img);
                Some(ctx.load_texture(
                    name,
                    color_image,
                    egui::TextureOptions::default()
                ))
            }
            Err(e) => {
                eprintln!("Failed to load image {}: {}", path, e);
                None
            }
        }
    }
    
    fn dynamic_image_to_color_image(&self, img: DynamicImage) -> ColorImage {
        let size = [img.width() as usize, img.height() as usize];
        let img_rgba = img.to_rgba8();
        let pixels = img_rgba.as_flat_samples();
        
        ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
    }
}