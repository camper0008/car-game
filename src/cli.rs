pub use clap::Parser;
use log::LevelFilter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value_t = 128.0)]
    pub mouse_sensitivity: f64,

    #[arg(short, long, default_value_t = false)]
    pub windowed: bool,

    #[arg(short, long, default_value_t = LevelFilter::Info)]
    pub log_level: LevelFilter,
}
