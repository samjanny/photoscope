# PhotoScope

Strumento di confronto visuale per immagini duplicate che permette di scegliere la migliore qualità.

## Funzionalità

- **Confronto visuale affiancato**: visualizza due immagini con lo stesso nome una accanto all'altra
- **Analisi qualità automatica**: valuta dimensione file, risoluzione (megapixel) e metadata EXIF
- **Interfaccia grafica intuitiva**: selezione rapida con tasti 1/2 o click
- **Copia automatica**: salva l'immagine scelta nella cartella `output/`

## Installazione

```bash
cargo build --release
```

## Utilizzo

### Modalità interattiva (GUI)
```bash
photoscope cartella1 cartella2
```

### Modalità automatica
Seleziona automaticamente l'immagine con qualità migliore:
```bash
photoscope cartella1 cartella2 --auto
```

## Esempio

```bash
# Confronta immagini tra due cartelle
photoscope C:\Foto\Originali C:\Foto\Backup

# Modalità automatica
photoscope C:\Foto\Originali C:\Foto\Backup --auto
```

## Controlli GUI

- **Tasto 1**: Seleziona immagine dalla prima cartella
- **Tasto 2**: Seleziona immagine dalla seconda cartella  
- **Tasto S**: Salta la coppia corrente
- **ESC**: Esci dall'applicazione

## Output

Le immagini selezionate vengono copiate nella cartella `output/` nella directory corrente.

## Formati supportati

- JPEG/JPG
- PNG
- GIF
- BMP
- TIFF/TIF
- WebP
- RAW (CR2, NEF, ARW, DNG)