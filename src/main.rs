const OFFSETX: usize = 50;
const OFFSETY: usize = 50;
const PX: usize = 200;
const PY: usize = 150;
const HOST: &str = "151.217.30.160:1234";
const THREADS: usize = 32;

use std::{
    io::Write,
    mem,
    net::TcpStream,
    ops::Range,
    sync::{Arc, Barrier, RwLock},
    thread,
};

use captrs::{Bgr8, Capturer};

fn main() {
    let mut cap = Capturer::new(0).expect("failed to create capturer");

    cap.capture_store_frame().expect("failed to capture frame");

    let (gx, gy) = cap.geometry();

    let factorx = gx as f32 / PX as f32;
    let factory = gy as f32 / PY as f32;

    let frame = vec![unsafe { mem::zeroed() }; gx as usize * gy as usize];
    let frame = Arc::new(RwLock::new(frame));

    let vsync = Arc::new(Barrier::new(THREADS + 1));

    let mut handles = Vec::new();
    for i in 0..THREADS {
        let shared_frame = frame.clone();
        let vsync = vsync.clone();

        let starty = (PY as f32 / THREADS as f32) as usize * i;
        let endy = (PY as f32 / THREADS as f32) as usize * (i + 1);

        let handle = thread::spawn(move || {
            painter(
                shared_frame.clone(),
                vsync,
                factorx,
                factory,
                gx,
                starty..endy,
            );
        });

        handles.push(handle);
    }

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
    frame: Arc<RwLock<Vec<Bgr8>>>,
    vsync: Arc<Barrier>,
    factorx: f32,
    factory: f32,
    gx: u32,
    y_range: Range<usize>,
) {
    let mut stream = TcpStream::connect(HOST).expect("failed to connect");

    loop {
        vsync.wait();

        let frame = frame.read().unwrap();

        for y in y_range.clone() {
            for x in 0..PX {
                let framex = (x as f32 * factorx) as u32;
                let framey = (y as f32 * factory) as u32;
                let pos = framey * gx + framex;

                let pix = frame[pos as usize];

                let x = x + OFFSETX;
                let y = y + OFFSETY;

                let msg = &[
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
                ];

                stream.write_all(msg).expect("failed to write pixel");
            }
        }
    }
}
