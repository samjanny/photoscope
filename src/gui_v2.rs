use crate::file_manager::FileManager;
use crate::image_analyzer::ImageAnalysis;
use anyhow::Result;
use eframe::egui;
use egui::{Color32, ColorImage, Context, FontId, Frame, Margin, RichText, CornerRadius, Stroke, TextureHandle, Vec2, Visuals};
use egui_phosphor::regular;
use image::{DynamicImage, GenericImageView, imageops::FilterType};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

const MAX_TEXTURE_SIZE: u32 = 2048;

// Colori del tema
const BG_COLOR: Color32 = Color32::from_rgb(24, 26, 31);
const CARD_BG: Color32 = Color32::from_rgb(32, 34, 41);
const CARD_HOVER: Color32 = Color32::from_rgb(38, 40, 48);
const ACCENT_BLUE: Color32 = Color32::from_rgb(59, 130, 246);
const ACCENT_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
const ACCENT_ORANGE: Color32 = Color32::from_rgb(251, 146, 60);
const DANGER_RED: Color32 = Color32::from_rgb(239, 68, 68);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(229, 231, 235);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);
const GOLD_STAR: Color32 = Color32::from_rgb(250, 204, 21);

#[derive(Clone)]
enum AppState {
    ShowingImages,
    Loading(String),
    ProcessingChoice(u8, PathBuf),
}

pub struct PhotoComparisonApp {
    // Stato dell'app
    state: AppState,
    
    // Tutte le coppie di file
    all_pairs: Vec<(PathBuf, PathBuf)>,
    current_index: usize,
    
    // Analisi correnti
    current_analysis1: Option<ImageAnalysis>,
    current_analysis2: Option<ImageAnalysis>,
    texture1: Option<TextureHandle>,
    texture2: Option<TextureHandle>,
    
    // File manager
    file_manager: FileManager,
    
    // Thread communication
    loading_message: Arc<Mutex<Option<String>>>,
    next_data: Arc<Mutex<Option<(ImageAnalysis, ImageAnalysis, DynamicImage, DynamicImage)>>>,
    
    // Statistiche
    selected_count: Arc<Mutex<usize>>,
    skipped_count: Arc<Mutex<usize>>,
    
    // Flags
    exit_program: bool,
    
    // UI state
    hover_image1: bool,
    hover_image2: bool,
    animation_time: f32,
    
    // Metadata transfer state
    metadata_transfer_source: Option<PathBuf>,
    metadata_transfer_pending: bool,
}

impl PhotoComparisonApp {
    pub fn new(
        pairs: Vec<(PathBuf, PathBuf)>,
        file_manager: FileManager,
    ) -> Self {
        PhotoComparisonApp {
            state: AppState::Loading("Caricamento prima coppia...".to_string()),
            all_pairs: pairs,
            current_index: 0,
            current_analysis1: None,
            current_analysis2: None,
            texture1: None,
            texture2: None,
            file_manager,
            loading_message: Arc::new(Mutex::new(None)),
            next_data: Arc::new(Mutex::new(None)),
            selected_count: Arc::new(Mutex::new(0)),
            skipped_count: Arc::new(Mutex::new(0)),
            exit_program: false,
            hover_image1: false,
            hover_image2: false,
            animation_time: 0.0,
            metadata_transfer_source: None,
            metadata_transfer_pending: false,
        }
    }
    
    pub fn run(mut self) -> Result<(usize, usize)> {
        let final_selected = self.selected_count.clone();
        let final_skipped = self.skipped_count.clone();
        
        if !self.all_pairs.is_empty() {
            self.load_current_pair();
        }
        
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_fullscreen(true)
                .with_title("PhotoScope Pro - Image Comparison Tool")
                .with_icon(Self::create_icon()),
            ..Default::default()
        };
        
        eframe::run_simple_native("PhotoScope Pro", options, move |ctx, _frame| {
            self.setup_custom_style(ctx);
            self.update(ctx);
            
            if self.exit_program {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }).map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;
        
        Ok((*final_selected.lock().unwrap(), *final_skipped.lock().unwrap()))
    }
    
    fn create_icon() -> egui::IconData {
        egui::IconData {
            rgba: vec![0; 32 * 32 * 4],
            width: 32,
            height: 32,
        }
    }
    
    fn setup_custom_style(&self, ctx: &Context) {
        // Initialize Phosphor fonts
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ctx.set_fonts(fonts);
        
        let mut style = (*ctx.style()).clone();
        
        // Font sizes
        style.text_styles.insert(
            egui::TextStyle::Heading,
            FontId::proportional(24.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::proportional(16.0),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            FontId::proportional(18.0),
        );
        
        // Spacing
        style.spacing.item_spacing = Vec2::new(12.0, 8.0);
        style.spacing.button_padding = Vec2::new(16.0, 10.0);
        style.spacing.indent = 20.0;
        
        // Visual tweaks
        style.visuals = Visuals::dark();
        style.visuals.window_fill = BG_COLOR;
        style.visuals.panel_fill = BG_COLOR;
        style.visuals.extreme_bg_color = CARD_BG;
        style.visuals.widgets.noninteractive.bg_fill = CARD_BG;
        style.visuals.widgets.inactive.bg_fill = CARD_BG;
        style.visuals.widgets.hovered.bg_fill = CARD_HOVER;
        style.visuals.widgets.active.bg_fill = ACCENT_BLUE;
        style.visuals.selection.bg_fill = ACCENT_BLUE;
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 8,
            spread: 0,
            color: Color32::from_black_alpha(48),
        };
        style.visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 2],
            blur: 6,
            spread: 0,
            color: Color32::from_black_alpha(48),
        };
        // Window rounding and widget rounding are handled differently in egui 0.32
        
        ctx.set_style(style);
    }
    
    fn update(&mut self, ctx: &Context) {
        // Non più necessario con fullscreen impostato nelle opzioni
        
        self.animation_time += ctx.input(|i| i.unstable_dt);
        
        // Controlla se ci sono nuovi dati dal thread
        if let Some((analysis1, analysis2, img1, img2)) = self.next_data.lock().unwrap().take() {
            self.current_analysis1 = Some(analysis1);
            self.current_analysis2 = Some(analysis2);
            self.texture1 = self.image_to_texture(ctx, img1, "img1");
            self.texture2 = self.image_to_texture(ctx, img2, "img2");
            self.state = AppState::ShowingImages;
        }
        
        match self.state.clone() {
            AppState::Loading(msg) => {
                self.show_loading_ui(ctx, &msg);
            }
            AppState::ShowingImages => {
                self.show_comparison_ui(ctx);
            }
            AppState::ProcessingChoice(choice, path) => {
                self.process_choice(choice, path);
                self.show_loading_ui(ctx, "Elaborazione scelta...");
            }
        }
        
        if matches!(self.state, AppState::Loading(_) | AppState::ProcessingChoice(_, _)) {
            ctx.request_repaint();
        }
    }
    
    fn show_comparison_ui(&mut self, ctx: &Context) {
        // Header principale compatto
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(3.0);
            self.show_modern_header(ui);
            ui.add_space(3.0);
        });
        
        // Footer con controlli compatto
        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.add_space(3.0);
            self.show_modern_controls(ui);
            ui.add_space(3.0);
        });
        
        // Area principale con immagini
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_modern_images(ui);
        });
        
        self.handle_keyboard_input(ctx);
    }
    
    fn show_modern_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Titolo compatto
            ui.label(RichText::new("PhotoScope Pro").size(18.0).color(TEXT_PRIMARY).strong());
            ui.separator();
            
            // Progress inline
            let progress = (self.current_index as f32) / (self.all_pairs.len() as f32);
            ui.add(egui::ProgressBar::new(progress)
                .desired_width(200.0)
                .text(format!("{}/{}", self.current_index + 1, self.all_pairs.len())));
            
            ui.separator();
            
            // Stats compatti
            ui.label(RichText::new(format!("{} {} | {} {} | Total: {}",
                regular::CHECK,
                *self.selected_count.lock().unwrap(),
                regular::ARROW_RIGHT,
                *self.skipped_count.lock().unwrap(),
                self.all_pairs.len())).size(14.0).color(TEXT_SECONDARY));
            
            // Show metadata transfer indicator if pending
            if self.metadata_transfer_pending {
                ui.separator();
                ui.label(RichText::new(format!("{} Metadati pronti per trasferimento", regular::SWAP))
                    .size(14.0)
                    .color(ACCENT_GREEN)
                    .strong());
            }
        });
    }
    
    fn show_modern_images(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        // Calcolo corretto considerando tutti gli spazi: 15px sinistra + 20px centro + 15px destra
        let total_spacing = 15.0 + 20.0 + 15.0;
        let card_width = (available_width - total_spacing) / 2.0;
        
        let (analysis1, analysis2, texture1, texture2) = match (&self.current_analysis1, &self.current_analysis2) {
            (Some(a1), Some(a2)) => (a1.clone(), a2.clone(), self.texture1.clone(), self.texture2.clone()),
            _ => return,
        };
        
        let quality_1_better = analysis1.quality_score >= analysis2.quality_score;
        let quality_2_better = analysis2.quality_score > analysis1.quality_score;
        let hover1 = self.hover_image1;
        let hover2 = self.hover_image2;
        
        // Prima riga: le immagini affiancate
        ui.horizontal(|ui| {
            ui.set_max_width(available_width);
            ui.add_space(15.0);
            
            // Immagine 1
            self.show_image_card(ui, 1, analysis1.clone(), texture1, card_width, 
                hover1, quality_1_better);
            
            ui.add_space(20.0);
            
            // Immagine 2
            self.show_image_card(ui, 2, analysis2.clone(), texture2, card_width,
                hover2, quality_2_better);
            
            ui.add_space(15.0);
        });
        
        // Seconda riga: i metadati (se presenti) sotto le immagini
        if !analysis1.exif_data.is_empty() || !analysis2.exif_data.is_empty() {
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.set_max_width(available_width);
                ui.add_space(15.0);
                
                // Metadati immagine 1 (o spazio vuoto per allineamento)
                ui.vertical(|ui| {
                    ui.set_max_width(card_width);
                    if !analysis1.exif_data.is_empty() {
                        self.show_metadata_card(ui, &analysis1.exif_data, card_width);
                    } else {
                        // Spazio vuoto per mantenere allineamento
                        ui.allocate_space(Vec2::new(card_width, 0.0));
                    }
                });
                
                ui.add_space(20.0);
                
                // Metadati immagine 2 (o spazio vuoto per allineamento)
                ui.vertical(|ui| {
                    ui.set_max_width(card_width);
                    if !analysis2.exif_data.is_empty() {
                        self.show_metadata_card(ui, &analysis2.exif_data, card_width);
                    } else {
                        // Spazio vuoto per mantenere allineamento
                        ui.allocate_space(Vec2::new(card_width, 0.0));
                    }
                });
                
                ui.add_space(15.0);
            });
        }
    }
    
    fn show_image_card(&mut self, ui: &mut egui::Ui, 
                       num: u8, 
                       analysis: ImageAnalysis, 
                       texture: Option<TextureHandle>,
                       width: f32,
                       is_hovered: bool,
                       is_best: bool) {
        ui.vertical(|ui| {
            ui.set_max_width(width);
            
            // Card container
            let card_bg = if is_hovered { CARD_HOVER } else { CARD_BG };
            Frame::NONE
                .fill(card_bg)
                .corner_radius(CornerRadius::same(12))
                .stroke(if is_best { 
                    Stroke::new(2.0, ACCENT_GREEN)
                } else { 
                    Stroke::new(1.0, Color32::from_gray(50))
                })
                .shadow(egui::epaint::Shadow {
                    offset: [0, if is_hovered { 4 } else { 2 }],
                    blur: if is_hovered { 12 } else { 4 },
                    spread: 0,
                    color: Color32::from_black_alpha(60),
                })
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    // Header minimo della card
                    ui.horizontal(|ui| {
                        let color = if num == 1 { ACCENT_BLUE } else { ACCENT_ORANGE };
                        
                        // Ottieni il nome del file
                        let filename = Path::new(&analysis.file_path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();
                        
                        // Tronca il nome se troppo lungo
                        let max_chars = 30;
                        let display_name = if filename.len() > max_chars {
                            format!("...{}", &filename[filename.len().saturating_sub(max_chars-3)..])
                        } else {
                            filename.to_string()
                        };
                        
                        // Crea il label con troncamento
                        let label_text = format!("[{}] {}", num, display_name);
                        let label = egui::Label::new(
                            RichText::new(label_text)
                                .size(14.0)
                                .color(color)
                        ).truncate();
                        
                        let response = ui.add(label);
                        
                        // Mostra tooltip con nome completo se troncato
                        if filename.len() > max_chars {
                            response.on_hover_text(filename.to_string());
                        }
                        
                        // Check if this image is the metadata source
                        let is_metadata_source = self.metadata_transfer_pending && 
                            self.metadata_transfer_source.as_ref()
                                .map(|p| p == Path::new(&analysis.file_path))
                                .unwrap_or(false);
                        
                        if is_metadata_source {
                            ui.label(RichText::new(format!(" {} META SORGENTE", regular::DATABASE)).color(ACCENT_GREEN).strong());
                        } else if is_best {
                            ui.label(RichText::new(format!(" {} MIGLIORE", regular::STAR)).color(ACCENT_GREEN).strong());
                        }
                    });
                    
                    // Info compatte su una riga con dimensioni e percentuale qualità
                    ui.label(RichText::new(format!("{}×{} | {:.1}MP | {:.1}MB | {} ({}%) {}",
                        analysis.width,
                        analysis.height,
                        analysis.megapixels,
                        analysis.file_size_mb,
                        analysis.get_quality_stars(),
                        analysis.quality_score,
                        if analysis.metadata_count > 0 { format!("| {} meta", analysis.metadata_count) } else { String::new() }
                    )).size(12.0).color(TEXT_SECONDARY));
                    
                    ui.add_space(4.0);
                    
                    // Area immagine - altezza fissa per tutte
                    let image_height = 600.0;
                    // Consideriamo i margini interni della card (16px * 2) e del frame immagine (8px * 2)
                    let image_width = width - 32.0 - 16.0;
                    
                    Frame::NONE
                        .fill(Color32::from_gray(20))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            ui.set_min_height(image_height);
                            ui.set_min_width(image_width);
                            
                            if let Some(texture) = texture {
                                let size = texture.size_vec2();
                                let scale_x = image_width / size.x;
                                let scale_y = image_height / size.y;
                                let scale = scale_x.min(scale_y);
                                let scaled_size = Vec2::new(size.x * scale, size.y * scale);
                                
                                // Centra l'immagine nell'area disponibile
                                let x_offset = (image_width - scaled_size.x) / 2.0;
                                let y_offset = (image_height - scaled_size.y) / 2.0;
                                
                                ui.add_space(y_offset.max(0.0));
                                ui.horizontal(|ui| {
                                    ui.add_space(x_offset.max(0.0));
                                    let response = ui.image((texture.id(), scaled_size));
                                    
                                    if num == 1 {
                                        self.hover_image1 = response.hovered();
                                    } else {
                                        self.hover_image2 = response.hovered();
                                    }
                                });
                            } else {
                                // Mostra spinner centrato
                                ui.add_space(image_height / 2.0 - 20.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(image_width / 2.0 - 20.0);
                                    ui.spinner();
                                });
                            }
                        });
                });
        });
    }
    
    
    fn show_metadata_card(&self, ui: &mut egui::Ui, exif_data: &Vec<(String, String)>, width: f32) {
        // Calcola l'altezza disponibile
        let available_height = ui.available_height();
        
        // Usa un'altezza fissa se lo spazio disponibile è troppo piccolo
        let card_height = if available_height > 200.0 {
            available_height - 10.0
        } else {
            200.0 // Altezza minima garantita
        };
        
        Frame::NONE
            .fill(CARD_BG)
            .corner_radius(CornerRadius::same(12))
            .stroke(Stroke::new(1.0, Color32::from_gray(50)))
            .shadow(egui::epaint::Shadow {
                offset: [0, 2],
                blur: 4,
                spread: 0,
                color: Color32::from_black_alpha(40),
            })
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_min_width(width - 24.0);
                ui.set_max_width(width - 24.0);
                ui.set_min_height(card_height - 24.0);
                
                // Titolo
                ui.label(RichText::new("Metadati EXIF").size(13.0).color(TEXT_PRIMARY).strong());
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                
                // Area scrollabile per i metadati
                let scroll_height = (card_height - 60.0).max(100.0);
                egui::ScrollArea::vertical()
                    .max_height(scroll_height)
                    .auto_shrink([false, false]) // Impedisce lo shrink automatico
                    .show(ui, |ui| {
                        for (key, value) in exif_data {
                            let formatted_key = key.replace("(", "").replace(")", "");
                            ui.horizontal(|ui| {
                                // Usa una larghezza fissa per la chiave per allineamento
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                    ui.set_min_width(150.0);
                                    ui.label(RichText::new(format!("{}:", formatted_key))
                                        .size(11.0)
                                        .color(TEXT_SECONDARY));
                                });
                                ui.label(RichText::new(value)
                                    .size(11.0)
                                    .color(TEXT_PRIMARY));
                            });
                        }
                        
                        // Aggiungi un po' di spazio alla fine per miglior leggibilità
                        ui.add_space(10.0);
                    });
            });
    }
    
    fn show_modern_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Pulsanti principali compatti
            let btn_size = Vec2::new(120.0, 35.0);
            
            if self.modern_button(ui, &format!("{} Prima (A)", regular::ARROW_LEFT), ACCENT_BLUE, btn_size) {
                self.make_choice(1);
            }
            
            if self.modern_button(ui, &format!("{} Seconda (D)", regular::ARROW_RIGHT), ACCENT_ORANGE, btn_size) {
                self.make_choice(2);
            }
            
            if self.modern_button(ui, &format!("{} Salta (S)", regular::ARROW_DOWN), TEXT_SECONDARY, btn_size) {
                self.skip_current();
            }
            
            if self.modern_button(ui, &format!("{} Meta (W)", regular::ARROW_UP), ACCENT_GREEN, btn_size) {
                self.transfer_metadata();
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.modern_button(ui, &format!("{} Esci", regular::X), DANGER_RED, btn_size) {
                    self.exit_program = true;
                }
                
                // Shortcuts help compatto
                ui.label(RichText::new(format!("{} A, D, S, W, ESC", regular::KEYBOARD)).size(12.0).color(TEXT_SECONDARY));
            });
        });
    }
    
    fn modern_button(&self, ui: &mut egui::Ui, text: &str, color: Color32, size: Vec2) -> bool {
        let button = egui::Button::new(RichText::new(text).size(18.0))
            .min_size(size)
            .fill(color.gamma_multiply(0.2))
            .stroke(Stroke::new(1.0, color));
        
        let response = ui.add(button);
        
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        
        response.clicked()
    }
    
    fn show_loading_ui(&self, ctx: &Context, message: &str) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let available_height = ui.available_height();
                ui.add_space(available_height / 2.0 - 100.0);
                
                // Animated spinner
                ui.spinner();
                ui.add_space(30.0);
                
                ui.heading(RichText::new(message).size(24.0).color(TEXT_PRIMARY));
                
                ui.add_space(20.0);
                
                // Progress info
                Frame::NONE
                    .fill(CARD_BG)
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(20, 12))
                    .show(ui, |ui| {
                        ui.label(RichText::new(format!("{} File {}/{}", regular::FILE, 
                            self.current_index + 1, self.all_pairs.len()))
                            .size(18.0)
                            .color(TEXT_SECONDARY));
                    });
            });
        });
    }
    
    fn handle_keyboard_input(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::A)) {
            self.make_choice(1);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::D)) {
            self.make_choice(2);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.skip_current();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::W)) {
            self.transfer_metadata();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.exit_program = true;
        }
    }
    
    fn make_choice(&mut self, choice: u8) {
        if let Some((path1, path2)) = self.all_pairs.get(self.current_index) {
            let path = if choice == 1 { path1.clone() } else { path2.clone() };
            self.state = AppState::ProcessingChoice(choice, path);
        }
    }
    
    fn skip_current(&mut self) {
        *self.skipped_count.lock().unwrap() += 1;
        self.move_to_next();
    }
    
    fn process_choice(&mut self, _choice: u8, path: PathBuf) {
        let file_manager = self.file_manager.clone();
        let next_data = self.next_data.clone();
        let pairs = self.all_pairs.clone();
        let next_index = self.current_index + 1;
        
        // Check if there's pending metadata transfer
        let metadata_source = if self.metadata_transfer_pending {
            self.metadata_transfer_source.clone()
        } else {
            None
        };
        
        // Clear metadata transfer state after using it
        self.metadata_transfer_pending = false;
        self.metadata_transfer_source = None;
        
        thread::spawn(move || {
            // Copy to output with optional metadata transfer
            if let Ok(_dest) = file_manager.copy_to_output_with_metadata(&path, metadata_source.as_deref()) {
                // Successo copia (e eventuale trasferimento metadati)
            }
            
            if next_index < pairs.len() {
                let (path1, path2) = &pairs[next_index];
                if let (Ok(a1), Ok(a2)) = (
                    ImageAnalysis::analyze_image(path1),
                    ImageAnalysis::analyze_image(path2)
                ) {
                    if let (Ok(img1), Ok(img2)) = (
                        Self::load_and_resize_image(path1),
                        Self::load_and_resize_image(path2)
                    ) {
                        *next_data.lock().unwrap() = Some((a1, a2, img1, img2));
                    }
                }
            }
        });
        
        *self.selected_count.lock().unwrap() += 1;
        self.state = AppState::Loading("Preparazione prossima coppia...".to_string());
        self.move_to_next();
    }
    
    fn move_to_next(&mut self) {
        self.current_index += 1;
        
        if self.current_index >= self.all_pairs.len() {
            self.exit_program = true;
            return;
        }
        
        if matches!(self.state, AppState::ShowingImages) {
            self.state = AppState::Loading("Caricamento...".to_string());
            self.load_current_pair();
        }
    }
    
    fn load_current_pair(&mut self) {
        if let Some((path1, path2)) = self.all_pairs.get(self.current_index) {
            let path1 = path1.clone();
            let path2 = path2.clone();
            let next_data = self.next_data.clone();
            
            thread::spawn(move || {
                if let (Ok(a1), Ok(a2)) = (
                    ImageAnalysis::analyze_image(&path1),
                    ImageAnalysis::analyze_image(&path2)
                ) {
                    if let (Ok(img1), Ok(img2)) = (
                        Self::load_and_resize_image(&path1),
                        Self::load_and_resize_image(&path2)
                    ) {
                        *next_data.lock().unwrap() = Some((a1, a2, img1, img2));
                    }
                }
            });
        }
    }
    
    fn load_and_resize_image(path: &Path) -> Result<DynamicImage> {
        let mut img = image::open(path)?;
        let (width, height) = img.dimensions();
        if width > MAX_TEXTURE_SIZE || height > MAX_TEXTURE_SIZE {
            let ratio = (MAX_TEXTURE_SIZE as f32 / width.max(height) as f32).min(1.0);
            let new_width = (width as f32 * ratio) as u32;
            let new_height = (height as f32 * ratio) as u32;
            img = img.resize(new_width, new_height, FilterType::Lanczos3);
        }
        Ok(img)
    }
    
    fn image_to_texture(&self, ctx: &Context, img: DynamicImage, name: &str) -> Option<TextureHandle> {
        let size = [img.width() as usize, img.height() as usize];
        let img_rgba = img.to_rgba8();
        let pixels = img_rgba.as_flat_samples();
        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        
        Some(ctx.load_texture(
            name,
            color_image,
            egui::TextureOptions::default()
        ))
    }
    
    fn transfer_metadata(&mut self) {
        // Get the current pair of files
        if let Some((path1, path2)) = self.all_pairs.get(self.current_index) {
            // Determine which image has more metadata
            let metadata_count_1 = self.current_analysis1.as_ref().map(|a| a.metadata_count).unwrap_or(0);
            let metadata_count_2 = self.current_analysis2.as_ref().map(|a| a.metadata_count).unwrap_or(0);
            
            if metadata_count_1 > metadata_count_2 {
                self.metadata_transfer_source = Some(path1.clone());
                self.metadata_transfer_pending = true;
                self.state = AppState::Loading(format!(
                    "Metadati marcati per trasferimento: immagine 1 ({} meta) → immagine selezionata", 
                    metadata_count_1
                ));
            } else if metadata_count_2 > metadata_count_1 {
                self.metadata_transfer_source = Some(path2.clone());
                self.metadata_transfer_pending = true;
                self.state = AppState::Loading(format!(
                    "Metadati marcati per trasferimento: immagine 2 ({} meta) → immagine selezionata", 
                    metadata_count_2
                ));
            } else if metadata_count_1 > 0 {
                // If both have same metadata count (and not zero), don't transfer
                self.state = AppState::Loading("Entrambe le immagini hanno già lo stesso numero di metadati".to_string());
                self.metadata_transfer_pending = false;
                self.metadata_transfer_source = None;
            } else {
                // Both have no metadata
                self.state = AppState::Loading("Nessuna immagine ha metadati da trasferire".to_string());
                self.metadata_transfer_pending = false;
                self.metadata_transfer_source = None;
            }
            
            // Show the state briefly, then return to showing images
            let next_data = self.next_data.clone();
            let pairs = self.all_pairs.clone();
            let current_index = self.current_index;
            
            thread::spawn(move || {
                // Wait a bit to show the message
                std::thread::sleep(std::time::Duration::from_millis(1500));
                
                // Reload current pair to go back to showing images
                if let Some((path1, path2)) = pairs.get(current_index) {
                    if let (Ok(a1), Ok(a2)) = (
                        ImageAnalysis::analyze_image(path1),
                        ImageAnalysis::analyze_image(path2)
                    ) {
                        if let (Ok(img1), Ok(img2)) = (
                            PhotoComparisonApp::load_and_resize_image(path1),
                            PhotoComparisonApp::load_and_resize_image(path2)
                        ) {
                            *next_data.lock().unwrap() = Some((a1, a2, img1, img2));
                        }
                    }
                }
            });
        }
    }
}