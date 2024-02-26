//! # Music Organizer
//! Organize your songs into folders! This crate can be used as a library and as a binary CLI
//! tool. The main and only function to use is [organize_songs](organize_songs).
//!
//! # Example
//!
//! This simple example specifies a [AppConfig](AppConfig) and orgranizes songs using
//! the [organize_songs](organize_songs) function.
//!
//! ```
//! use music_organizer::{self, AppConfig};
//!
//! let app_config = AppConfig::default();
//!
//! music_organizer::organize_songs(app_config).unwrap();
//! ```

use std::{path::PathBuf, collections::HashMap, fs};
use clap::Parser;
use id3::TagLike;
use rand::prelude::*;

pub enum OperationMode {
    Organize,
    Shuffle
}

/// Arguments for use in the binary. 
/// The recommended way to use them is through the [Args::parse_and_get_dirs](Args::parse_and_get_dirs) function.
///
/// Implemented using the [clap](https://docs.rs/clap/4.3.22/clap/index.html) crate.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input directory to process
    #[arg(short, long)]
    input_dir: Option<PathBuf>,
    /// Output directory to save organized files to
    #[arg(short, long)]
    output_dir: Option<PathBuf>,
    /// Overwrite contents of a directory
    #[arg(short = 'O', long)]
    overwrite: bool,
    /// Enable shuffle mode
    #[arg(short = 'S', long)]
    shuffle: bool
}

/// Config for use with the [organize_songs](organize_songs) function.
/// In a binary, you can use the [Args::parse_and_get_dirs](Args::parse_and_get_dirs) function to 
/// get a AppConfig from arguments passed to the binary.
pub struct AppConfig {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub overwrite: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let input_dir = std::env::current_dir().expect("couldn't get the current working directory");
        let mut output_dir = input_dir.clone();
        output_dir.push("output");

        Self { input_dir, output_dir, overwrite: false }
    }
}

impl Args {
    /// Parses the arguments automatically, gets input and output directories outputs them in a
    /// [AppConfig](AppConfig).
    /// 
    /// If the input directory is omitted, the current working directory is used.
    ///
    /// If the output directory is omitted, the `output/` directory inside the current working
    /// directory is used (and created is neccessary).
    ///
    /// # Example
    ///
    /// This example gets the [AppConfig](AppConfig) using 
    /// the [Args::parse_and_get_dirs](Args::parse_and_get_dirs) function and
    /// prints the directories to console.
    ///
    /// ```
    /// use music_organizer::{Args, AppConfig};
    /// 
    /// let app_config = Args::parse_and_get_dirs();
    /// let (input_dir, output_dir) = app_config;
    ///
    /// println!("Input directory: {}.", input_dir.to_str());
    /// println!("Output directory: {}.", output_dir.to_str());
    /// ```
    pub fn parse_and_get_dirs() -> AppConfig {
        let args = Args::parse();

        let input_dir = if let Some(dir) = args.input_dir {
            dir
        } else {
            std::env::current_dir().expect("couldn't get current working directory")
        };

        let output_dir = if let Some(dir) = args.output_dir {
            dir
        } else {
            let mut dir = input_dir.clone();
            dir.push("output");
            dir
        };

        let overwrite = args.overwrite;

        AppConfig { input_dir, output_dir, overwrite }
    }

    pub fn parse_and_get_mode_and_dirs() -> (AppConfig, OperationMode) {
        let args = Args::parse();

        let input_dir = if let Some(dir) = args.input_dir {
            dir
        } else {
            std::env::current_dir().expect("couldn't get current working directory")
        };

        let output_dir = if let Some(dir) = args.output_dir {
            dir
        } else {
            let mut dir = input_dir.clone();
            dir.push("output");
            dir
        };

        let overwrite = args.overwrite;
        let operation_mode = if args.shuffle { OperationMode::Shuffle } else { OperationMode::Organize };

        (AppConfig { input_dir, output_dir, overwrite }, operation_mode)
    }
}

fn create_sanitization_regex() -> regex::Regex {
    regex::Regex::new(r#"<?>?:?"?/?\\?\|?\??\*?\x00?"#).unwrap()
}

/// Organizes songs from input directory and copies them to the output directory.
///
/// # Implementation
///
/// The function gets IDv3 tags from MP3 files using the `[id3](https://docs.rs/id3/1.7.0/id3/index.html)` crate
/// and copies the organized songs to the output directory.
///
/// # Example
///
/// This example organizes songs using the arguments supplied to the binary.
/// ```
/// use music_organizer::{self, Args};
///
/// let app_config = Args::parse_and_get_dirs();
///
/// music_organizer::organize_songs(app_config).unwrap();
/// ```
///
/// # Errors
///
/// This function may return an IO error as [specified in the std crate](https://doc.rust-lang.org/stable/std/io/type.Result.html).
pub fn organize_songs(app_config: AppConfig) -> std::io::Result<()> {
    let songs = get_songs_from_path(app_config.input_dir.clone()).unwrap();
    println!("Found {} songs.", songs.len());

    let grouped_paths = group_song_list_by_artist(songs);

    output_grouped_songs(app_config, grouped_paths)
}

/// Shuffles songs from input directory and copies them with prepended random number in the file name to the output directory.
///
/// # Example
///
/// This example shuffles songs using the arguments supplied to the binary.
/// ```
/// use music_organizer::{self, Args};
///
/// let app_config = Args::parse_and_get_dirs();
///
/// music_organizer::shuffle_songs(app_config).unwrap();
/// ```
///
/// # Errors
///
/// This function may return an IO error as [specified in the std crate](https://doc.rust-lang.org/stable/std/io/type.Result.html).
pub fn shuffle_songs(app_config: AppConfig) -> std::io::Result<()> {
    let mut songs = get_songs_from_path(app_config.input_dir.clone()).unwrap();
    println!("Found {} songs.", songs.len());

    let mut rng = rand::thread_rng();
    songs.shuffle(&mut rng);

    output_shuffled_songs(app_config, songs)
}

fn create_output_dir(app_config: &AppConfig) -> std::io::Result<()> {
    let output_dir = &app_config.output_dir;

    if let Err(e) = fs::create_dir(&output_dir) {
        use std::io::ErrorKind::*;
        match e.kind() {
            AlreadyExists => println!("Output directory already exists... Continuing."),
            _ => return Err(e)
        }
    }

    Ok(())
}

fn output_shuffled_songs(app_config: AppConfig, shuffled_songs: Vec<PathBuf>) -> std::io::Result<()> {
    let AppConfig { output_dir, overwrite, .. } = &app_config;

    create_output_dir(&app_config)?;

    for (i, path) in shuffled_songs.into_iter().enumerate() {
        let mut output_song_path = output_dir.clone();
        let file_name = path.file_name().expect("filename should've been well-formed");
        let file_name_str = file_name.to_str().expect("OsStr should've been well-formed");
        println!("processing: {}", file_name_str);

        let new_file_name = format!("{} - {}", i, file_name_str);

        output_song_path.push(new_file_name);

        if fs::metadata(&output_song_path).is_ok() {
            if *overwrite {
                fs::remove_file(&output_song_path)?;
                fs::copy(&path, &output_song_path)?;
            } else {
                println!("File at path `{}` already exists! Use --overwrite to permit overwriting.", output_song_path.to_str().unwrap());
            }
        } else {
            fs::copy(&path, &output_song_path)?;
        }
    }

    Ok(())
}

type SongList = Vec<PathBuf>;

type GroupedSongs = HashMap<String, SongList>;

fn output_grouped_songs(app_config: AppConfig, grouped_songs: GroupedSongs) -> std::io::Result<()> {
    let AppConfig { input_dir, output_dir, overwrite } = &app_config;

    create_output_dir(&app_config)?;

    for (artist, paths) in grouped_songs {
        let mut new_dir_path = output_dir.clone();
        new_dir_path.push(&artist);
        if let Err(e) = fs::create_dir(&new_dir_path) {
            use std::io::ErrorKind::*;
            match e.kind() {
                AlreadyExists => println!("Directory for artist {} already exists... Continuing.", artist),
                _ => return Err(e),
            }
        }

        for path in paths {
            let file_name = path.file_name().expect("filename should've been well-formed");
            println!("processing: {}", file_name.to_str().unwrap());

            let mut input_path = input_dir.clone();
            input_path.push(&file_name);

            let mut output_path = new_dir_path.clone();
            output_path.push(&file_name);

            if fs::metadata(&output_path).is_ok() {
                if *overwrite {
                    fs::remove_file(&output_path)?;
                    fs::copy(&input_path, &output_path)?;
                } else {
                    println!("File at path `{}` already exists! Use --overwrite to permit overwriting.", output_path.to_str().unwrap());
                }
            } else {
                fs::copy(&input_path, &output_path)?;
            }
        }
    }
    
    Ok(())
}

fn group_song_list_by_artist(song_list: SongList) -> GroupedSongs {
    let mut organized_paths = HashMap::new();
    let regex = create_sanitization_regex();

    for song_path in song_list.into_iter() {
        let artist = {
            let str = get_artist_from_file(&song_path).unwrap_or("Unknown".to_owned());
            String::from(regex.replace_all(&str, ""))
        };

        organized_paths.entry(artist).or_insert(Vec::new()).push(song_path);
    }

    organized_paths
}

fn get_artist_from_file(file_path: &PathBuf) -> Option<String> {
    match id3::Tag::read_from_path(file_path) {
        Ok(tag) => tag.artist().map(|s| s.to_owned()),
        Err(_) => None
    }
}

fn get_songs_from_path(path: PathBuf) -> Result<SongList, std::io::Error> {
    let dir_entries = std::fs::read_dir(path)?;
    
    let song_paths: Vec<PathBuf> = dir_entries
        .map(|res| res.expect("couldn't read file"))
        .filter(|entry| {
            if !entry.path().is_file() { return false; }
            let file_name = entry
                .file_name();
            let extension = file_name
                .to_str()
                .expect("file_name wasn't valid to convert to a str")
                .split(".")
                .last();
            match extension
            {
                Some(ext) => ext == "mp3",
                None => false
            }
        })
        .map(|entry| entry.path())
        .collect();

    Ok(song_paths)
}
