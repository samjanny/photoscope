use anyhow::Result;
use eframe::egui;
use egui::{Color32, Frame, Margin, RichText, CornerRadius, Stroke, Vec2, Visuals, FontId};
use egui_phosphor::regular;
use rfd::FileDialog;
use std::path::PathBuf;

// Colori del tema (consistenti con gui_v2.rs)
const BG_COLOR: Color32 = Color32::from_rgb(24, 26, 31);
const CARD_BG: Color32 = Color32::from_rgb(32, 34, 41);
const CARD_HOVER: Color32 = Color32::from_rgb(38, 40, 48);
const ACCENT_BLUE: Color32 = Color32::from_rgb(59, 130, 246);
const ACCENT_GREEN: Color32 = Color32::from_rgb(34, 197, 94);
const DANGER_RED: Color32 = Color32::from_rgb(239, 68, 68);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(229, 231, 235);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);
const WARNING_YELLOW: Color32 = Color32::from_rgb(251, 146, 60);

pub struct FolderSelectorApp {
    folder1: Option<PathBuf>,
    folder2: Option<PathBuf>,
    folders_selected: bool,
}

impl FolderSelectorApp {
    pub fn new() -> Self {
        FolderSelectorApp {
            folder1: None,
            folder2: None,
            folders_selected: false,
        }
    }
    
    pub fn run(mut self) -> Result<Option<(PathBuf, PathBuf)>> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([900.0, 600.0])
                .with_title("PhotoScope Pro - Setup"),
            ..Default::default()
        };
        
        let result = std::sync::Arc::new(std::sync::Mutex::new(None));
        let result_clone = result.clone();
        
        eframe::run_simple_native("PhotoScope Setup", options, move |ctx, _frame| {
            self.setup_custom_style(ctx);
            self.update(ctx);
            
            if self.folders_selected {
                if let (Some(f1), Some(f2)) = (&self.folder1, &self.folder2) {
                    *result_clone.lock().unwrap() = Some((f1.clone(), f2.clone()));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }).map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;
        
        Ok(result.lock().unwrap().clone())
    }
    
    fn setup_custom_style(&self, ctx: &egui::Context) {
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
        // Window rounding and widget rounding are handled differently in egui 0.32
        
        ctx.set_style(style);
    }
    
    fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                
                // Header
                self.show_header(ui);
                
                ui.add_space(40.0);
                
                // Folder selection cards
                Frame::NONE
                    .inner_margin(Margin::symmetric(40, 0))
                    .show(ui, |ui| {
                        ui.set_max_width(800.0);
                        
                        // Folder 1 Card
                        self.show_folder_card(ui, 1);
                        
                        ui.add_space(20.0);
                        
                        // Folder 2 Card
                        self.show_folder_card(ui, 2);
                        
                        ui.add_space(30.0);
                        
                        ui.add_space(40.0);
                        
                        // Action buttons
                        self.show_actions(ui, ctx);
                    });
            });
        });
    }
    
    fn show_header(&self, ui: &mut egui::Ui) {
        ui.heading(RichText::new(format!("{} PhotoScope Pro", regular::APERTURE)).size(32.0).color(TEXT_PRIMARY));
        ui.add_space(8.0);
        ui.label(RichText::new("Professional Image Comparison Tool").size(18.0).color(TEXT_SECONDARY));
        ui.add_space(12.0);
        ui.label(RichText::new("Seleziona le cartelle da confrontare per trovare le migliori versioni delle tue immagini")
            .size(16.0)
            .color(TEXT_SECONDARY));
    }
    
    fn show_folder_card(&mut self, ui: &mut egui::Ui, num: u8) {
        let folder_ref = if num == 1 { &self.folder1 } else { &self.folder2 };
        let has_folder = folder_ref.is_some();
        let folder_path = folder_ref.as_ref().and_then(|p| p.to_str()).unwrap_or("Nessuna cartella selezionata");
        let color = if num == 1 { ACCENT_BLUE } else { Color32::from_rgb(251, 146, 60) };
        
        let mut new_path = None;
        
        Frame::NONE
            .fill(CARD_BG)
            .corner_radius(CornerRadius::same(12))
            .stroke(if has_folder {
                Stroke::new(2.0, color)
            } else {
                Stroke::new(1.0, Color32::from_gray(50))
            })
            .inner_margin(Margin::same(20))
            .shadow(egui::epaint::Shadow {
                offset: [0, 2],
                blur: 4,
                spread: 0,
                color: Color32::from_black_alpha(40),
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Folder number and label
                    ui.label(RichText::new(format!("{} Cartella {}", regular::FOLDER, num))
                        .size(20.0)
                        .color(color)
                        .strong());
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Select button
                        if self.modern_button(ui, &format!("{} Seleziona", regular::FOLDER_OPEN), color, Vec2::new(120.0, 35.0)) {
                            if let Some(path) = FileDialog::new()
                                .set_title(&format!("Seleziona cartella {}", num))
                                .pick_folder()
                            {
                                new_path = Some(path);
                            }
                        }
                    });
                });
                
                ui.add_space(12.0);
                
                // Path display
                Frame::NONE
                    .fill(Color32::from_gray(20))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_min_height(30.0);
                        
                        let text_color = if has_folder {
                            ACCENT_GREEN
                        } else {
                            TEXT_SECONDARY
                        };
                        
                        ui.label(RichText::new(folder_path)
                            .color(text_color)
                            .monospace());
                    });
            });
        
        // Update folder after the frame
        if let Some(path) = new_path {
            if num == 1 {
                self.folder1 = Some(path);
            } else {
                self.folder2 = Some(path);
            }
        }
    }
    
    fn show_actions(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let both_selected = self.folder1.is_some() && self.folder2.is_some();
        
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 320.0) / 2.0);
            
            // Start button
            ui.add_enabled_ui(both_selected, |ui| {
                let btn_color = if both_selected { ACCENT_GREEN } else { Color32::from_gray(80) };
                if self.modern_button(ui, &format!("{} Avvia Confronto", regular::PLAY), btn_color, Vec2::new(150.0, 45.0)) {
                    self.folders_selected = true;
                }
            });
            
            ui.add_space(20.0);
            
            // Exit button
            if self.modern_button(ui, &format!("{} Esci", regular::X), DANGER_RED, Vec2::new(150.0, 45.0)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        
        if !both_selected {
            ui.add_space(20.0);
            Frame::NONE
                .fill(WARNING_YELLOW.gamma_multiply(0.2))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(16, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(regular::WARNING.to_string()).color(WARNING_YELLOW).size(20.0));
                        ui.label(RichText::new("Seleziona entrambe le cartelle per continuare")
                            .color(WARNING_YELLOW));
                    });
                });
        }
    }
    
    fn modern_button(&self, ui: &mut egui::Ui, text: &str, color: Color32, size: Vec2) -> bool {
        let button = egui::Button::new(RichText::new(text).size(16.0))
            .min_size(size)
            .fill(color.gamma_multiply(0.2))
            .stroke(Stroke::new(1.0, color));
        
        let response = ui.add(button);
        
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        
        response.clicked()
    }
}