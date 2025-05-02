mod app;
mod command;
mod storage;
mod todo;

use app::App;
use color_eyre::Result;
use ratatui::init as tui_init;
use ratatui::restore as tui_restore;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = tui_init();
    let res = App::new().run(terminal);
    tui_restore();
    res
}
