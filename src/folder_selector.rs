use anyhow::Result;
use eframe::egui;
use egui::Color32;
use rfd::FileDialog;
use std::path::PathBuf;

pub struct FolderSelectorApp {
    folder1: Option<PathBuf>,
    folder2: Option<PathBuf>,
    folders_selected: bool,
    auto_mode: bool,
}

impl FolderSelectorApp {
    pub fn new() -> Self {
        FolderSelectorApp {
            folder1: None,
            folder2: None,
            folders_selected: false,
            auto_mode: false,
        }
    }
    
    pub fn run(mut self) -> Result<Option<(PathBuf, PathBuf, bool)>> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([700.0, 400.0])
                .with_title("PhotoScope - Seleziona Cartelle"),
            ..Default::default()
        };
        
        let result = std::sync::Arc::new(std::sync::Mutex::new(None));
        let result_clone = result.clone();
        
        eframe::run_simple_native("PhotoScope Folder Selector", options, move |ctx, _frame| {
            self.update(ctx);
            
            if self.folders_selected {
                if let (Some(f1), Some(f2)) = (&self.folder1, &self.folder2) {
                    *result_clone.lock().unwrap() = Some((f1.clone(), f2.clone(), self.auto_mode));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }).map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;
        
        Ok(result.lock().unwrap().clone())
    }
    
    fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                ui.heading("PhotoScope - Confronto Immagini");
                
                ui.add_space(10.0);
                ui.label("Seleziona le due cartelle da confrontare per trovare immagini duplicate");
                
                ui.add_space(30.0);
                
                ui.horizontal(|ui| {
                    ui.set_min_width(650.0);
                    
                    ui.vertical(|ui| {
                        ui.label("Cartella 1:");
                        ui.horizontal(|ui| {
                            let folder_text = self.folder1
                                .as_ref()
                                .and_then(|p| p.to_str())
                                .unwrap_or("Nessuna cartella selezionata");
                            
                            let text_color = if self.folder1.is_some() {
                                Color32::from_rgb(100, 200, 100)
                            } else {
                                Color32::from_rgb(150, 150, 150)
                            };
                            
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(folder_text)
                                        .color(text_color)
                                        .monospace()
                                )
                                .wrap()
                            );
                        });
                        
                        if ui.button("📁 Seleziona Cartella 1").clicked() {
                            if let Some(path) = FileDialog::new()
                                .set_title("Seleziona la prima cartella")
                                .pick_folder()
                            {
                                self.folder1 = Some(path);
                            }
                        }
                    });
                });
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    ui.set_min_width(650.0);
                    
                    ui.vertical(|ui| {
                        ui.label("Cartella 2:");
                        ui.horizontal(|ui| {
                            let folder_text = self.folder2
                                .as_ref()
                                .and_then(|p| p.to_str())
                                .unwrap_or("Nessuna cartella selezionata");
                            
                            let text_color = if self.folder2.is_some() {
                                Color32::from_rgb(100, 200, 100)
                            } else {
                                Color32::from_rgb(150, 150, 150)
                            };
                            
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(folder_text)
                                        .color(text_color)
                                        .monospace()
                                )
                                .wrap()
                            );
                        });
                        
                        if ui.button("📁 Seleziona Cartella 2").clicked() {
                            if let Some(path) = FileDialog::new()
                                .set_title("Seleziona la seconda cartella")
                                .pick_folder()
                            {
                                self.folder2 = Some(path);
                            }
                        }
                    });
                });
                
                ui.add_space(30.0);
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.auto_mode, "Modalità automatica (sceglie sempre la migliore qualità)");
                });
                
                ui.add_space(20.0);
                
                let both_selected = self.folder1.is_some() && self.folder2.is_some();
                
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(both_selected, |ui| {
                        if ui.button(
                            egui::RichText::new("▶ Avvia Confronto")
                                .size(16.0)
                                .color(if both_selected {
                                    Color32::from_rgb(100, 200, 255)
                                } else {
                                    Color32::from_rgb(100, 100, 100)
                                })
                        ).clicked() {
                            self.folders_selected = true;
                        }
                    });
                    
                    ui.add_space(20.0);
                    
                    if ui.button(
                        egui::RichText::new("✖ Esci")
                            .size(16.0)
                            .color(Color32::from_rgb(255, 100, 100))
                    ).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                if !both_selected {
                    ui.add_space(20.0);
                    ui.colored_label(
                        Color32::from_rgb(255, 200, 100),
                        "⚠ Seleziona entrambe le cartelle per continuare"
                    );
                }
            });
        });
    }
}