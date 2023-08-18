# Music Organizer
Organize your songs into folders! This crate can be used as a library and as a binary CLI
tool. The main and only function to use is `organize_songs`.

# Installation

There are no prebuilt binaries at the moment, so you'll have to use Cargo to install this application. Install Cargo using [rustup](https://rustup.rs/).

To install Music Organizer using Cargo, open up a terminal window and run this command:
```sh
cargo install --git https://github.com/catink123/music-organizer.git
```

# Example

This simple example specifies an `AppConfig` and orgranizes songs using
the `organize_songs` function.

```rust
use music_organizer::{self, AppConfig};

let app_config = AppConfig::default();

music_organizer::organize_songs(app_config).unwrap();
```
