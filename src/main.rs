pub mod args;

// The default buffer size for reading the client stream.
// - Big enough so we don't have to expand
// - Small enough to not take up to much memory
const CMD_READ_BUFFER_SIZE: usize = 32;

// The response format of the screen size from a pixelflut server.
const PIX_SERVER_SIZE_REGEX: &str = r"^(?i)\s*SIZE\s+([[:digit:]]+)\s+([[:digit:]]+)\s*$";

use std::{
    io::{BufRead, Error, ErrorKind, Write},
    mem,
    net::TcpStream,
    ops::Range,
    sync::{Arc, Barrier, RwLock},
    thread,
};

use args::ArgHandler;
use bufstream::BufStream;
use captrs::{Bgr8, Capturer};
use regex::Regex;

fn main() {
    let args = Arc::new(ArgHandler::parse());

    println!("Starting screen capture");

    let cap = Capturer::new(args.screen()).expect("failed to create capturer");

    // Gather facts about the host
    let screen_size =
        gather_host_facts(&args).expect("Failed to gather facts about pixelflut server");

    let (gx, gy) = cap.geometry();

    let (outx, outy) = args.size(Some(screen_size));

    let factorx = gx as f32 / outx as f32;
    let factory = gy as f32 / outy as f32;

    let frame = vec![unsafe { mem::zeroed() }; gx as usize * gy as usize];
    let frame = Arc::new(RwLock::new(frame));

    let thread_count = args.count();
    let vsync = Arc::new(Barrier::new(thread_count + 1));

    for i in 0..thread_count as u16 {
        let args = args.clone();
        let frame = frame.clone();
        let vsync = vsync.clone();

        let starty = (outy as f32 / thread_count as f32) as u16 * i;
        let endy = (outy as f32 / thread_count as f32) as u16 * (i + 1);

        thread::spawn(move || {
            painter(
                args,
                frame,
                vsync,
                (factorx, factory),
                outx,
                gx,
                starty..endy,
            );
        });
    }

    println!("Streaming now... (use CTRL+C to stop)");

    capturer(cap, frame, vsync);
}

fn capturer(mut cap: Capturer, shared_frame: Arc<RwLock<Vec<Bgr8>>>, vsync: Arc<Barrier>) {
    loop {
        cap.capture_store_frame().expect("failed to capture frame");
        shared_frame
            .write()
            .unwrap()
            .copy_from_slice(cap.get_stored_frame().unwrap());

        // Synchronize with painters
        vsync.wait();
    }
}

fn painter(
    args: Arc<ArgHandler>,
    frame: Arc<RwLock<Vec<Bgr8>>>,
    vsync: Arc<Barrier>,
    (factorx, factory): (f32, f32),
    outx: u16,
    gx: u32,
    y_range: Range<u16>,
) {
    let binary = args.binary();
    let flush = !args.no_flush();

    let mut stream = TcpStream::connect(args.host()).expect("failed to connect");

    loop {
        // Wait for other painters and frame to be captured
        vsync.wait();

        let frame = frame.read().unwrap();

        for y in y_range.clone() {
            for x in 0..outx {
                let framex = (x as f32 * factorx) as u32;
                let framey = (y as f32 * factory) as u32;
                let pos = framey * gx + framex;

                let pix = frame[pos as usize];

                let x = x + args.offset().0;
                let y = y + args.offset().1;

                // Send pixel in binary or text mode
                if binary {
                    stream
                        .write_all(&[
                            b'P',
                            b'B',
                            x as u8,
                            (x >> 8) as u8,
                            y as u8,
                            (y >> 8) as u8,
                            pix.r,
                            pix.g,
                            pix.b,
                            255,
                        ])
                        .expect("failed to write pixel");
                } else {
                    let msg = format!("PX {x} {y} {:02X}{:02X}{:02X}\n", pix.r, pix.g, pix.b);
                    stream
                        .write_all(msg.as_bytes())
                        .expect("failed to write pixel");
                }

                // Flush stream
                if flush {
                    stream.flush().expect("failed to flush stream");
                }
            }
        }
    }
}

/// Gather important facts about the host.
fn gather_host_facts(args: &ArgHandler) -> Result<(u16, u16), Error> {
    let mut stream = BufStream::new(TcpStream::connect(args.host()).expect("failed to connect"));
    stream
        .write_all("SIZE\n".as_bytes())
        .expect("failed to request SIZE from screen");
    stream
        .flush()
        .expect("failed to flush stream to request screen size");

    // Build a regex to parse the screen size
    let re = Regex::new(PIX_SERVER_SIZE_REGEX).unwrap();

    // Read the output
    // TODO: this operation may get stuck (?) if nothing is received from the server
    let mut response = String::with_capacity(CMD_READ_BUFFER_SIZE);
    stream.read_line(&mut response)?;

    // Find captures in the data, return the result
    let size = match re.captures(&response) {
        Some(matches) => (
            matches[1]
                .parse::<u16>()
                .expect("Failed to parse screen width, received malformed data"),
            matches[2]
                .parse::<u16>()
                .expect("Failed to parse screen height, received malformed data"),
        ),
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to parse screen size, received malformed data",
            ))
        }
    };

    // Print status
    println!("Gathered screen size: {}x{}", size.0, size.1);

    Ok(size)
}
