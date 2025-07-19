use nix::unistd::Uid;
use tecarius::{app::App, config, error, pacman::Pacman};

#[tokio::main]
async fn main() -> error::Result<()> {
    // Get color configuration
    let theme_colors = config::theme_colors().unwrap_or_default();

    // Check super-user rights
    if !Uid::effective().is_root() {
        eprintln!("Tecarius must be run with root permissions.");
        return Err(error::Error::SuperUserError);
    }

    // Init pacman
    let pacman = Pacman::new()?;

    // Init terminal
    let mut terminal = ratatui::init();

    // Init and run application
    let app = App::new(pacman, theme_colors);
    let app_result = app.run(&mut terminal);

    // Restore terminal
    ratatui::restore();

    app_result
}
