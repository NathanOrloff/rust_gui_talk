use gui_talk::MyApp;
use eframe::egui::Vec2;

use std::net::{TcpListener, TcpStream};
use std::io::{Write, Error, self, BufRead, BufReader};
use std::thread;
use polling::{Event, Poller};
use std::time::Duration;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::TryRecvError;

fn main() {

    let port = std::env::args().nth(1).unwrap_or("".to_string());
    let ip = std::env::args().nth(2).unwrap_or("".to_string());
    
    if ip.chars().count() > 0 && port.chars().count() > 0 {
        client(port, ip);
    } else if port.chars().count() > 0 {
        server(port)  
    } else {
        println!("No port given!");
    }
}

fn server(port: String){
    let listener = TcpListener::bind(format!("0.0.0.0:{}",port)).expect("could not bind");
    for stream in listener.incoming() {
        match stream {
            Err(e) => { eprintln!("failed: {}", e) },
            Ok(stream) => {
                accept(stream).unwrap_or_else(|error| eprintln!("{:?}", error))
            },
        };
    }

}

fn client(port: String, ip: String){
    let mut stream = TcpStream::connect(format!("{}:{}", ip, port)).expect("could not connect to host");
    println!("Connecting to {ip}:{port} please wait...");
    loop {
        let mut buffer: Vec<u8> = Vec::new();
        let mut response = BufReader::new(&stream);
        response.read_until(b'\n', &mut buffer).expect("could not read into buffer");

        if buffer[0] == b'y' || buffer[0] == b'Y' {
            println!("Connection accepted");
            let app = MyApp::new(stream);
            let mut options = eframe::NativeOptions::default();
            options.initial_window_size = Some(Vec2::new(800., 400.));
            eframe::run_native(
                "Talk",
                options,
                Box::new(|_cc| Box::new(app)),
            );
            return
        } else {
            println!("Connection rejected... exiting program");
            std::process::exit(0x1);
        }
    }
}

fn accept(mut stream: TcpStream) -> Result<(), Error> {
    println!("Incoming connection from: {}", stream.peer_addr()?);
    let mut input = String::new();
    print!("Accept incoming connection (y/n)? ");
    io::stdout().flush().expect("flush failed!");

    io::stdin().read_line(&mut input).expect("could not read input");
    stream.write(input.as_bytes()).expect("Failed to write to client");

    if input.chars().nth(0).unwrap() == 'y' || input.chars().nth(0).unwrap() == 'Y' {
        let app = MyApp::new(stream);
        let mut options = eframe::NativeOptions::default();
        options.initial_window_size = Some(Vec2::new(800., 400.));
        eframe::run_native(
            "Talk",
            options,
            Box::new(|_cc| Box::new(app)),
        );

    } else {
        return Ok(());
    }
    return Ok(());
}