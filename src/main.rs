use music_organizer::Args;

fn main() -> std::io::Result<()> {
    let dir_config = Args::parse_and_get_dirs();

    music_organizer::organize_songs(dir_config)?;

    Ok(())
}
