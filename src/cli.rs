use std::env;
use std::path::PathBuf;

pub struct CliArgs {
    pub disk: Option<String>,
    pub tracks: Option<Vec<u8>>,
    pub out_dir: PathBuf,
    pub skip_artwork: bool,
}

pub fn parse_cli_args() -> CliArgs {
    let mut disk: Option<String> = None;
    let mut tracks: Option<Vec<u8>> = None;
    let mut out: Option<String> = None;
    let mut skip_artwork = false;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "---help" || arg == "-h" {
            print_help();
            std::process::exit(0);
        }

        if let Some(value) = arg.strip_prefix("--disk=") {
            disk = Some(value.to_string());
            continue;
        }

        if arg == "--disk" {
            let Some(value) = args.next() else {
                exit_with_usage_error("Expected a value after --disk");
            };

            disk = Some(value);
            continue;
        }

        if let Some(value) = arg.strip_prefix("--track=") {
            tracks = Some(parse_tracks(value));
            continue;
        }

        if arg == "--track" {
            let Some(value) = args.next() else {
                exit_with_usage_error("Expected a value after --track");
            };

            tracks = Some(parse_tracks(&value));
            continue;
        }

        if let Some(value) = arg.strip_prefix("--out=") {
            out = Some(value.to_string());
            continue;
        }

        if arg == "--out" {
            let Some(value) = args.next() else {
                exit_with_usage_error("Expected a value after --out");
            };

            out = Some(value);
            continue;
        }

        if arg == "--skip-artwork" {
            skip_artwork = true;
            continue;
        }

        exit_with_usage_error(&format!("Unknown argument: {arg}"));
    }

    let out_dir = resolve_out_dir(out.as_deref());

    CliArgs {
        disk,
        tracks,
        out_dir,
        skip_artwork,
    }
}

fn parse_tracks(value: &str) -> Vec<u8> {
    let mut tracks = Vec::new();

    for part in value.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Ok(track) = trimmed.parse::<u8>() else {
            exit_with_usage_error(&format!("Invalid track number in --track: {trimmed}"));
        };

        if track > 99 {
            exit_with_usage_error(&format!(
                "Track number out of range in --track: {track} (expected 0..=99)"
            ));
        }

        tracks.push(track);
    }

    if tracks.is_empty() {
        exit_with_usage_error("--track was provided but no valid track numbers were found");
    }

    tracks.sort();
    tracks.dedup();
    tracks
}

fn resolve_out_dir(out: Option<&str>) -> PathBuf {
    let path = match out {
        Some(raw_path) => PathBuf::from(raw_path),
        None => match env::current_dir() {
            Ok(current_dir) => current_dir,
            Err(error) => {
                println!("Failed to resolve current directory: {error}");
                std::process::exit(1);
            }
        },
    };

    if !path.exists() {
        exit_with_usage_error(&format!(
            "Output directory does not exist: {}",
            path.display()
        ));
    }

    if !path.is_dir() {
        exit_with_usage_error(&format!(
            "Output path is not a directory: {}",
            path.display()
        ));
    }

    path
}

fn print_help() {
    println!("Audio CD ripper");
    println!();
    println!("Usage:");
    println!("  cargo run -- [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --disk <PATH>        CD drive path (e.g. disk4, \\\\.\\E:, /dev/sr0)");
    println!("  --track <LIST>       Comma-separated track numbers to rip (0..=99), e.g. 1,2,5");
    println!("  --out <DIR>          Existing output directory (defaults to current directory)");
    println!("  --skip-artwork       Skip downloading album artwork");
    println!("  --help, -h           Show this help message");
}

fn exit_with_usage_error(message: &str) -> ! {
    println!("{message}");
    println!("Use --help to see available options.");
    std::process::exit(2);
}
