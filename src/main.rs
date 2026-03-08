use clap::Parser;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 📁 Automatically organize files into folders by type
#[derive(Parser, Debug)]
#[command(author = "Roseanne Park", version, about)]
struct Args {
    /// Target directory to organize
    #[arg(default_value = ".")]
    directory: String,

    /// Dry run — show what WOULD happen without moving files
    #[arg(short, long)]
    dry_run: bool,

    /// Include files in subdirectories
    #[arg(short, long)]
    recursive: bool,
}

/// Returns the category folder name for a given file extension
fn get_category(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" => "📷 Images",
        // Videos
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm"          => "🎬 Videos",
        // Audio
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a"                   => "🎵 Audio",
        // Documents
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx"        => "📄 Documents",
        // Code
        "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "java"
        | "rb" | "php" | "swift" | "kt" | "sol"                          => "💻 Code",
        // Web
        "html" | "css" | "jsx" | "tsx" | "vue" | "json" | "xml"         => "🌐 Web",
        // Archives
        "zip" | "tar" | "gz" | "rar" | "7z" | "bz2"                     => "📦 Archives",
        // Text
        "txt" | "md" | "csv" | "log" | "yaml" | "toml" | "env"          => "📝 Text",
        // Executables
        "exe" | "msi" | "deb" | "rpm" | "dmg" | "sh" | "bat"            => "⚙️  Executables",
        // Fonts
        "ttf" | "otf" | "woff" | "woff2"                                 => "🔤 Fonts",
        // Unknown
        _                                                                  => "❓ Others",
    }
}

fn collect_files(dir: &Path, recursive: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if recursive {
        // Use walkdir for recursive traversal
        for entry in walkdir::WalkDir::new(dir)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                files.push(entry.into_path());
            }
        }
    } else {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                }
            }
        }
    }

    files
}

fn main() {
    let args = Args::parse();

    let dir = Path::new(&args.directory);

    if !dir.exists() || !dir.is_dir() {
        eprintln!("{}", format!("Error: '{}' is not a valid directory.", args.directory).red());
        std::process::exit(1);
    }

    println!("\n{}", "📁 File Organizer — Rust".bold().cyan());
    println!("{}", "─".repeat(50).dimmed());
    println!("  Directory: {}", args.directory.yellow());
    if args.dry_run {
        println!("  Mode: {}", "🔍 DRY RUN (no files will be moved)".yellow());
    }
    println!("{}\n", "─".repeat(50).dimmed());

    let files = collect_files(dir, args.recursive);

    if files.is_empty() {
        println!("{}", "  No files found to organize.".dimmed());
        return;
    }

    // Group files by category
    let mut categorized: HashMap<&str, Vec<PathBuf>> = HashMap::new();

    for file in &files {
        // Skip hidden files
        if file
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        let ext = file
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");

        let category = get_category(ext);
        categorized.entry(category).or_default().push(file.clone());
    }

    let mut total_moved = 0;
    let mut total_skipped = 0;

    for (category, files) in &categorized {
        // Strip emoji prefix for folder name (use clean name)
        let folder_name = category
            .chars()
            .skip_while(|c| !c.is_alphabetic())
            .collect::<String>()
            .trim()
            .to_string();

        let dest_dir = dir.join(&folder_name);

        println!("  {} ({})", category.bold(), files.len().to_string().yellow());

        for file in files {
            let file_name = file.file_name().unwrap();
            let destination = dest_dir.join(file_name);

            print!(
                "    {} → {}/{}",
                file_name.to_string_lossy().white(),
                folder_name.dimmed(),
                file_name.to_string_lossy().dimmed()
            );

            if args.dry_run {
                println!(" {}", "[dry run]".yellow());
                total_moved += 1;
            } else {
                // Create destination directory if needed
                if !dest_dir.exists() {
                    if let Err(e) = fs::create_dir_all(&dest_dir) {
                        println!(" {}", format!("[error: {e}]").red());
                        total_skipped += 1;
                        continue;
                    }
                }

                // Skip if destination already exists
                if destination.exists() {
                    println!(" {}", "[skipped: exists]".dimmed());
                    total_skipped += 1;
                    continue;
                }

                match fs::rename(file, &destination) {
                    Ok(_) => {
                        println!(" {}", "✓".green());
                        total_moved += 1;
                    }
                    Err(e) => {
                        println!(" {}", format!("[error: {e}]").red());
                        total_skipped += 1;
                    }
                }
            }
        }

        println!();
    }

    println!("{}", "─".repeat(50).dimmed());
    println!(
        "  {} files {} | {} skipped\n",
        total_moved.to_string().green().bold(),
        if args.dry_run { "would be moved" } else { "moved" },
        total_skipped.to_string().yellow()
    );
}
