use music_organizer::{Args, organize_songs, shuffle_songs, OperationMode::*};

fn main() -> std::io::Result<()> {
    let (app_config, operation_mode) = Args::parse_and_get_mode_and_dirs();

    match operation_mode {
        Organize => organize_songs(app_config)?,
        Shuffle => shuffle_songs(app_config)?
    }

    Ok(())
}
