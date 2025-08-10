use anyhow::Result;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct LoadingGui {
    message: String,
    is_complete: Arc<Mutex<bool>>,
    start_time: Instant,
}

impl LoadingGui {
    pub fn new(message: String, is_complete: Arc<Mutex<bool>>) -> Self {
        LoadingGui {
            message,
            is_complete,
            start_time: Instant::now(),
        }
    }
    
    pub fn show(mut self) -> Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([450.0, 200.0])
                .with_title("PhotoScope - Caricamento")
                .with_resizable(false)
                .with_always_on_top(),
            ..Default::default()
        };
        
        eframe::run_simple_native("PhotoScope Loading", options, move |ctx, _frame| {
            // Controlla se il caricamento è completato
            if *self.is_complete.lock().unwrap() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
            
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    
                    // Spinner animato grande
                    ui.add(egui::Spinner::new().size(60.0));
                    
                    ui.add_space(20.0);
                    
                    // Messaggio principale
                    ui.heading(&self.message);
                    
                    ui.add_space(10.0);
                    
                    // Tempo trascorso
                    let elapsed = self.start_time.elapsed();
                    ui.label(format!("Tempo: {:.1}s", elapsed.as_secs_f32()));
                    
                    ui.add_space(10.0);
                    
                    // Messaggio secondario
                    ui.label("Attendere prego...");
                });
            });
            
            // Richiedi repaint continuo per aggiornare spinner e tempo
            ctx.request_repaint();
        }).map_err(|e| anyhow::anyhow!("Loading GUI error: {}", e))?;
        
        Ok(())
    }
}

// Funzione helper per eseguire un'operazione con GUI di loading
pub fn run_with_loading_gui<F, T>(message: &str, operation: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    use std::thread;
    use std::sync::mpsc;
    
    let is_complete = Arc::new(Mutex::new(false));
    let is_complete_clone = is_complete.clone();
    
    // Channel per ricevere il risultato
    let (tx, rx) = mpsc::channel();
    
    // Avvia il thread per l'operazione pesante
    thread::spawn(move || {
        let result = operation();
        tx.send(result).ok();
        *is_complete_clone.lock().unwrap() = true;
    });
    
    // Mostra la GUI di loading
    let loading_gui = LoadingGui::new(message.to_string(), is_complete.clone());
    
    // Avvia la GUI in un thread separato
    let gui_thread = thread::spawn(move || {
        loading_gui.show().ok();
    });
    
    // Aspetta il risultato
    let result = rx.recv()
        .map_err(|_| anyhow::anyhow!("Failed to receive result from worker thread"))?;
    
    // Assicurati che la GUI si chiuda
    *is_complete.lock().unwrap() = true;
    
    // Aspetta che il thread GUI finisca
    gui_thread.join().ok();
    
    result
}