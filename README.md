# pixelpwnr-cast

A quick [pixelflut][pixelflut] ([video][pixelflut-video]) client in
[Rust][rust] for use at [37C3][37C3], that streams your screen to pixelflut panels.

For a high performance pixelflut client and server implementation, see:
- [pixelpwnr (client)][pixelpwnr]
- [pixelpwnr-server (server)][pixelpwnr-server]

## Features

* Stream your desktop in real-time
* Many concurrent drawing pipes, fast multithreading
* Control over render sizes and offset
* Automatic image sizing and formatting
* Blazingly fast [binary protocol](https://github.com/timvisee/pixelpwnr-server#the-binary-px-command) (`PB` with `--binary`)
* Linux (X11), Windows and macOS

## Usage

Cast your desktop:
```bash
# Flut your screen
# - To host 127.0.0.1 on port 8080
# - With 4 painting threads
# - With the size of the screen (default)
pixelpwnr-cast 127.0.0.1:8080 -c 4
```

Cast to a small frame:
```bash
# Flut your screen to a small frame
# - To host 127.0.0.1 on port 8080
# - With 4 painting threads
# - With a size of (400, 300)
# - With an offset of (100, 100)
pixelpwnr-cast 127.0.0.1:8080 -c 4 -w 400 -h 300 -x 100 -y 100
```

Use the `--help` flag, or see the [help](#help) section for all available
options.

## Installation

For installation, Git and Rust cargo are required.
Install the latest version of Rust with [rustup][rustup].

Then, clone and install `pixelpwnr-cast` with:

```bash
# Clone the project
git clone https://github.com/timvisee/pixelpwnr-cast.git
cd pixelpwnr-cast

# Install pixelpwnr-cast
cargo install --path .

# Start using pixelpwnr-cast
pixelpwnr-cast --help

# or run it directly from Cargo
cargo run --release -- --help
```

Or just build it and invoke the binary directly (Linux/macOS):

```bash
# Clone the project
git clone https://github.com/timvisee/pixelpwnr-cast.git
cd pixelpwnr-cast

# Build the project (release version)
cargo build --release

# Start using pixelpwnr-cast
./target/release/pixelpwnr-cast --help
```

## Help

```text
pixelpwnr-cast --help

Insanely fast pixelflut client for casting your screen

Usage: pixelpwnr-cast [OPTIONS] --screen <SCREEN_ID> <HOST>

Arguments:
  <HOST>  The host to pwn "host:port"

Options:
      --help                Show this help
  -s, --screen <SCREEN_ID>  Screen number (X11 ID) [default: 0]
  -w, --width <PIXELS>      Draw width [default: screen width]
  -h, --height <PIXELS>     Draw height [default: screen height]
  -x <PIXELS>               Draw X offset [default: 0]
  -y <PIXELS>               Draw Y offset [default: 0]
  -c, --count <COUNT>       Number of concurrent threads [default: number of CPUs]
  -b, --binary              Use binary mode to set pixels (`PB` protocol extension) [default: off]
  -n, --no-flush            Do not flush socket after each pixel [default: on]
  -f, --frame-buffering     Whether to use frame buffering
  -V, --version             Print version
```

## Relevant projects

* [pixelpwnr (client)][pixelpwnr]
* [pixelpwnr-server (server)][pixelpwnr-server]

## License

This project is released under the GNU GPL-3.0 license.
Check out the [LICENSE](LICENSE) file for more information.


[37C3]: https://events.ccc.de/congress/2023/infos/startpage.html
[pixelflut]: https://cccgoe.de/wiki/Pixelflut
[pixelflut-video]: https://vimeo.com/92827556/
[pixelpwnr]: https://github.com/timvisee/pixelpwnr
[pixelpwnr-server]: https://github.com/timvisee/pixelpwnr-server
[rust]: https://www.rust-lang.org/
[rustup]: https://rustup.rs/
