pub mod args;

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

// Two frame buffers.
type Frames = Arc<(Arc<RwLock<Vec<Bgr8>>>, Arc<RwLock<Vec<Bgr8>>>)>;

// The default buffer size for reading the client stream.
// - Big enough so we don't have to expand
// - Small enough to not take up to much memory
const CMD_READ_BUFFER_SIZE: usize = 32;

// The response format of the screen size from a pixelflut server.
const PIX_SERVER_SIZE_REGEX: &str = r"^(?i)\s*SIZE\s+([[:digit:]]+)\s+([[:digit:]]+)\s*$";

fn main() {
    let args = Arc::new(ArgHandler::parse());

    println!("Starting screen capture");

    let cap = Capturer::new(args.screen()).expect("failed to create capturer");

    // Gather facts about the host
    let screen_size =
        gather_host_facts(&args).expect("Failed to gather facts about pixelflut server");

    let (cap_width, cap_height) = cap.geometry();
    let (server_width, server_height) = args.size(Some(screen_size));

    let factor_x = cap_width as f32 / server_width as f32;
    let factor_y = cap_height as f32 / server_height as f32;

    let base_frame = vec![unsafe { mem::zeroed() }; cap_width as usize * cap_height as usize];
    let cur_frame = Arc::new(RwLock::new(base_frame.clone()));
    let next_frame = Arc::new(RwLock::new(base_frame));

    let frames = Arc::new((cur_frame, next_frame));

    let thread_count = args.count();
    let vsync = Arc::new(Barrier::new(thread_count + 1));

    for i in 0..thread_count as u16 {
        let args = args.clone();
        let frames = frames.clone();
        let vsync = vsync.clone();

        let row_start = ((i as f32 / thread_count as f32) * server_height as f32) as u16;
        let row_end = (((i + 1) as f32 / thread_count as f32) * server_height as f32) as u16;

        thread::spawn(move || {
            painter(
                args,
                frames,
                vsync,
                (factor_x, factor_y),
                cap_width,
                server_width,
                row_start..row_end,
            );
        });
    }

    println!("Casting now... (use CTRL+C to stop)");

    capturer(cap, frames, vsync, args.frame_buffering());
}

fn capturer(mut cap: Capturer, frames: Frames, vsync: Arc<Barrier>, frame_buffering: bool) {
    loop {
        // Capture new frame
        cap.capture_store_frame().expect("failed to capture frame");

        if frame_buffering {
            // Write capture to new frame
            let mut new_frame = frames.1.write().unwrap();
            new_frame.copy_from_slice(cap.get_stored_frame().unwrap());

            // Swap new frame with current for upcoming paint
            let mut cur_frame = frames.0.write().unwrap();
            mem::swap(&mut *new_frame, &mut *cur_frame);
        } else {
            // Write capture directly to current frame
            let mut cur_frame = frames.0.write().unwrap();
            cur_frame.copy_from_slice(cap.get_stored_frame().unwrap());
        }

        // Synchronize with painters
        vsync.wait();
    }
}

fn painter(
    args: Arc<ArgHandler>,
    frame: Frames,
    vsync: Arc<Barrier>,
    (factor_x, factor_y): (f32, f32),
    cap_width: u32,
    server_width: u16,
    rows: Range<u16>,
) {
    let binary = args.binary();
    let flush = args.flush();
    let alpha = args.alpha();

    let mut stream = TcpStream::connect(args.host()).expect("failed to connect");

    loop {
        // Wait for other painters and frame to be captured
        vsync.wait();

        let frame = frame.0.read().unwrap();

        for y in rows.clone() {
            for x in 0..server_width {
                // Get current pixel from frame
                let frame_x = (x as f32 * factor_x) as u32;
                let frame_y = (y as f32 * factor_y) as u32;
                let frame_pos = frame_y * cap_width + frame_x;
                let pix = unsafe { frame.get_unchecked(frame_pos as usize) };

                // Pixel position on server
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
                            alpha,
                        ])
                        .expect("failed to write pixel");
                } else {
                    let msg = if alpha == u8::MAX {
                        format!("PX {x} {y} {:02X}{:02X}{:02X}\n", pix.r, pix.g, pix.b)
                    } else {
                        format!(
                            "PX {x} {y} {:02X}{:02X}{:02X}{alpha:02X}\n",
                            pix.r, pix.g, pix.b,
                        )
                    };
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
