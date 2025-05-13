use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Options {
    #[arg(short = 'l', long)]
    pub skip_long_tests: bool,

    #[arg(short, long, default_value_t = 64u32)]
    pub start_range: u32,

    #[arg(short, long, default_value_t = 1024u32)]
    pub finish_range: u32,

    #[arg(short = 't', long, default_value_t = 64u32)]
    pub step_range: u32,

    #[arg(short, long, default_value_t = 25.0)]
    pub max_spin_duration: f32, // ms
}

pub fn parse_options() -> Options {
    Options::parse()
}
