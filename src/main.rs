use nix::unistd::Uid;
use std::process::exit;
use tecarius::{app::App, config, error, pacman::Pacman};

#[tokio::main]
async fn main() -> error::Result<()> {
    // Get color configuration
    let theme_colors = config::theme_colors().unwrap_or_default();

    // Check super-user rights
    if !Uid::effective().is_root() {
        eprintln!("Tecarius must be run with root permissions.");
        exit(1);
    }

    let pacman = Pacman::new()?;
    let result = App::new(pacman, theme_colors)
        .run(&mut ratatui::init())
        .await;

    // Restore terminal
    ratatui::restore();

    result
}
