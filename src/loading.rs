use anyhow::Result;
use eframe::egui;

pub struct LoadingWindow {
    message: String,
}

impl LoadingWindow {
    pub fn new(message: String) -> Self {
        LoadingWindow { message }
    }
    
    pub fn show(&self) -> Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 200.0])
                .with_title("PhotoScope - Caricamento")
                .with_resizable(false)
                .with_decorations(false)
                .with_always_on_top(),
            ..Default::default()
        };
        
        let message = self.message.clone();
        
        eframe::run_simple_native("PhotoScope Loading", options, move |ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    
                    // Spinner animato
                    ui.add(egui::Spinner::new().size(50.0));
                    
                    ui.add_space(20.0);
                    
                    // Messaggio
                    ui.heading(&message);
                    
                    ui.add_space(10.0);
                    
                    ui.label("Attendere prego...");
                });
            });
            
            // Richiedi repaint continuo per animazione spinner
            ctx.request_repaint();
        }).map_err(|e| anyhow::anyhow!("Loading window error: {}", e))?;
        
        Ok(())
    }
}

// Funzione helper per mostrare brevemente lo spinner
pub fn show_loading_spinner(message: &str) {
    // Creiamo una versione semplificata che mostra solo brevemente lo spinner
    // Questa è più una notifica che una finestra bloccante
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 150.0])
            .with_title("PhotoScope")
            .with_resizable(false)
            .with_always_on_top()
            .with_position([500.0, 400.0]),
        ..Default::default()
    };
    
    let msg = message.to_string();
    let start_time = std::time::Instant::now();
    
    let _ = eframe::run_simple_native("PhotoScope Loading", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.add(egui::Spinner::new().size(40.0));
                ui.add_space(15.0);
                ui.label(&msg);
            });
        });
        
        // Auto-chiude dopo 0.5 secondi (giusto per mostrare che sta facendo qualcosa)
        if start_time.elapsed().as_millis() > 500 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        
        ctx.request_repaint();
    });
}