mod file_manager;
mod folder_selector;
mod gui;
mod gui_v2;
mod image_analyzer;
mod loading;
mod loading_gui;

use anyhow::Result;
use clap::Parser;
use colored::*;
use file_manager::FileManager;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "PhotoScope")]
#[command(about = "Confronta immagini duplicate e seleziona la migliore qualità", long_about = None)]
struct Args {
    #[arg(help = "Prima cartella da analizzare (opzionale se vuoi usare la GUI)")]
    folder1: Option<PathBuf>,
    
    #[arg(help = "Seconda cartella da analizzare (opzionale se vuoi usare la GUI)")]
    folder2: Option<PathBuf>,
    
    #[arg(short, long, help = "Modalità batch (salta conferma per ogni file)")]
    batch: bool,
    
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let (folder1, folder2, from_cli) = if args.folder1.is_none() || args.folder2.is_none() {
        println!("{}", "╔══════════════════════════════════════╗".bright_cyan());
        println!("{}", "║         PhotoScope v0.1.0            ║".bright_cyan());
        println!("{}", "║   Confronto e Selezione Immagini     ║".bright_cyan());
        println!("{}", "╚══════════════════════════════════════╝".bright_cyan());
        println!();
        println!("{} Apertura interfaccia di selezione cartelle...", "→".bright_green());
        
        let selector = folder_selector::FolderSelectorApp::new();
        match selector.run()? {
            Some((f1, f2)) => (f1, f2, false),
            None => {
                println!("{} Operazione annullata dall'utente.", "✗".bright_red());
                return Ok(());
            }
        }
    } else {
        let f1 = args.folder1.unwrap();
        let f2 = args.folder2.unwrap();
        (f1, f2, true)
    };
    
    if from_cli {
        println!("{}", "╔══════════════════════════════════════╗".bright_cyan());
        println!("{}", "║         PhotoScope v0.1.0            ║".bright_cyan());
        println!("{}", "║   Confronto e Selezione Immagini     ║".bright_cyan());
        println!("{}", "╚══════════════════════════════════════╝".bright_cyan());
        println!();
    }
    
    let file_manager = FileManager::new(folder1.clone(), folder2.clone())?;
    
    println!("{} Ricerca file con lo stesso nome...", "→".bright_green());
    let matching_files = file_manager.find_matching_files()?;
    
    if matching_files.is_empty() {
        println!("{} Nessun file con lo stesso nome trovato nelle due cartelle.", "✗".bright_red());
        return Ok(());
    }
    
    println!("{} Trovate {} coppie di file da confrontare", 
        "✓".bright_green(), 
        matching_files.len().to_string().bright_yellow());
    println!();
    
    // Usa la nuova GUI unificata
    println!("{} Avvio interfaccia grafica...", "→".bright_green());
    
    let app = gui_v2::PhotoComparisonApp::new(
        matching_files,
        file_manager,
    );
    
    let (selected_count, skipped_count) = app.run()?;
    
    println!("{}", "════════════════════════════════════════".bright_cyan());
    println!("{} Processo completato!", "✓".bright_green());
    println!("  {} File selezionati: {}", "•".bright_cyan(), selected_count.to_string().bright_green());
    println!("  {} File saltati: {}", "•".bright_cyan(), skipped_count.to_string().bright_yellow());
    println!("  {} Output salvato in: {}", "•".bright_cyan(), "output/".bright_white());
    
    Ok(())
}
