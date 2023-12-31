extern crate clap;
extern crate num_cpus;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, disable_help_flag = true)]
pub struct Arguments {
    // manually redefine help, but without short option, because `-h`
    // is already used by the height option.
    /// Show this help
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,

    /// The host to pwn "host:port"
    host: String,

    /// Screen number (X11 ID)
    #[arg(short, long, value_name = "SCREEN_ID", default_value_t = 0)]
    screen: usize,

    /// Draw width [default: screen width]
    #[arg(short, long, value_name = "PIXELS")]
    width: Option<u16>,
    /// Draw height [default: screen height]
    #[arg(short, long, value_name = "PIXELS")]
    height: Option<u16>,

    /// Draw X offset
    #[arg(short, value_name = "PIXELS", default_value_t = 0)]
    x: u16,
    /// Draw Y offset
    #[arg(short, value_name = "PIXELS", default_value_t = 0)]
    y: u16,

    /// Alpha channel [0, 255] [default: 255]
    #[arg(short, long, value_name = "ALPHA", default_value_t = 255)]
    alpha: u8,

    /// Number of concurrent threads [default: number of CPUs]
    #[arg(short, long, aliases = ["thread", "threads"])]
    count: Option<usize>,

    /// Use binary mode to set pixels (`PB` protocol extension) [default: off]
    #[arg(short, long, alias = "bin")]
    binary: bool,

    /// Flush socket after each pixel [default: true]
    #[arg(short, long, action = clap::ArgAction::Set, value_name = "ENABLED", default_value_t = true)]
    flush: bool,

    /// Whether to use frame buffering.
    #[arg(short, long, action = clap::ArgAction::Set, value_name = "ENABLED", default_value_t = true, alias = "frame-buf")]
    frame_buffering: bool,
}

/// CLI argument handler.
pub struct ArgHandler {
    data: Arguments,
}

impl ArgHandler {
    pub fn parse() -> ArgHandler {
        ArgHandler {
            data: Arguments::parse(),
        }
    }

    /// Get the host property.
    pub fn host(&self) -> &str {
        self.data.host.as_str()
    }

    /// Get the thread count.
    pub fn count(&self) -> usize {
        self.data.count.unwrap_or_else(num_cpus::get)
    }

    /// Get the screen ID.
    pub fn screen(&self) -> usize {
        self.data.screen
    }

    /// Get the image size.
    /// Use the given default value if not set.
    pub fn size(&self, def: Option<(u16, u16)>) -> (u16, u16) {
        (
            self.data
                .width
                .unwrap_or(def.expect("No screen width set or known").0),
            self.data
                .height
                .unwrap_or(def.expect("No screen height set or known").1),
        )
    }

    /// Get the image offset.
    pub fn offset(&self) -> (u16, u16) {
        (self.data.x, self.data.y)
    }

    /// Get the alpha channel value.
    pub fn alpha(&self) -> u8 {
        self.data.alpha
    }

    /// Whether to use binary mode.
    pub fn binary(&self) -> bool {
        self.data.binary
    }

    /// Whether to flush after each pixel.
    pub fn flush(&self) -> bool {
        self.data.flush
    }

    /// Whether to use frame buffering.
    pub fn frame_buffering(&self) -> bool {
        self.data.frame_buffering
    }
}
