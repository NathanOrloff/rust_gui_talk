use eframe::egui;
use chrono;

use std::net::{TcpListener, TcpStream};
use std::io::{Write, Error, self, BufRead, BufReader};
use std::thread;
use polling::{Event, Poller};
use std::time::Duration;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::TryRecvError;

const PADDING_SIDE_OUTER: f32 = 25.;
const PADDING_TOP_OUTER: f32 = 5.;
const PADDING_INNER: f32 = 5.;
const PADDING_BACKGROUND: f32 = 0.0;
const PADDING_TOP_BOT_PANEL: f32 = -30.;
const PADDING_TOP_TOP_PANEL: f32 = 20.;

const ROUNDING_FRONT_PANELS: f32 = 0.5;
const ROUNDING_BACKGROUND: f32 = 0.0;

const CHAT_HEIGHT: f32 = 290.;
const INPUT_HEIGHT: f32 = 20.;

const DARK_PURPLE: egui::Color32 = egui::Color32::from_rgb(77, 65, 71);



pub struct MyApp {
    input: String,
    chat_history: Vec<String>,
    stream: TcpStream,
    output: String,
    poller: Poller,
    key: usize,
}

impl MyApp {
    pub fn get_chat_history(&self) -> &Vec<String> {
        &self.chat_history
    }

    pub fn new(stream_new: TcpStream) -> Self {
        let key_new = 7;
        let new = Self {
                    input: "".to_owned(),
                    chat_history: Vec::new(),  
                    stream: stream_new,
                    output: "".to_owned(),
                    poller: Poller::new().unwrap(),
                    key: key_new,
                };
        
        new.poller.add(&new.stream, Event::readable(key_new));
        new.stream.set_nonblocking(true);
        return new;
    }
    
    fn talk(&mut self) -> Result<(), Error>{
        
        let mut events = Vec::new();
      
        events.clear();
        self.poller.wait(&mut events, Some(Duration::from_millis(10)))?;

        let mut buffer = String::new();
        let mut reader = BufReader::new(&self.stream);

        for ev in &events {
            if ev.key == self.key{
                match reader.read_line(&mut buffer){
                    Ok(num_bytes) => {
                        if num_bytes == 0 {
                            println!("---------------------Transmission Ended---------------------");
                            self.stream.shutdown(std::net::Shutdown::Both).expect("Failed to shutdown stream");
                            return Ok(()); 
                        }
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                        println!("---------------------Transmission Ended---------------------");
                        self.stream.shutdown(std::net::Shutdown::Both).expect("Failed to shutdown stream");
                        return Ok(());  
                    },
                };
                buffer.pop().unwrap();
                self.chat_history.push(buffer.clone());
                self.poller.modify(&self.stream, Event::readable(self.key))?;
            }
        }

        if self.output.chars().count() > 0 {
            self.stream.write(self.output.clone().as_bytes()).expect("Failed to write to stream");
            self.output = "".to_owned();
        }
        return Ok(());
        
    } 
    
}

impl eframe::App for MyApp {
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let background = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: PADDING_BACKGROUND, right: PADDING_BACKGROUND, top: PADDING_BACKGROUND, bottom: PADDING_BACKGROUND },
            outer_margin: egui::style::Margin { left: PADDING_BACKGROUND, right: PADDING_BACKGROUND, top: PADDING_BACKGROUND, bottom: PADDING_BACKGROUND },
            rounding: egui::Rounding { nw: ROUNDING_BACKGROUND, ne: ROUNDING_BACKGROUND, sw: ROUNDING_BACKGROUND, se: ROUNDING_BACKGROUND },
            shadow: eframe::epaint::Shadow { extrusion: 1.0, color: DARK_PURPLE },
            fill: DARK_PURPLE,
            stroke: egui::Stroke::new(2.0, DARK_PURPLE),
        };

        let mut panel = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: PADDING_INNER, right: PADDING_INNER, top: PADDING_INNER, bottom: PADDING_INNER },
            outer_margin: egui::style::Margin { left: PADDING_SIDE_OUTER, right: PADDING_SIDE_OUTER, top: PADDING_TOP_TOP_PANEL, bottom: PADDING_TOP_OUTER },
            rounding: egui::Rounding { nw: ROUNDING_FRONT_PANELS, ne: ROUNDING_FRONT_PANELS, sw: ROUNDING_FRONT_PANELS, se: ROUNDING_FRONT_PANELS },
            shadow: eframe::epaint::Shadow { extrusion: 1.0, color: egui::Color32::BLACK },
            fill: egui::Color32::DARK_GRAY,
            stroke: egui::Stroke::new(2.0, egui::Color32::BLACK),
        };


        egui::CentralPanel::default().frame(background).show(ctx, |_ui| {
            egui::TopBottomPanel::top("panel1").frame(panel).min_height(CHAT_HEIGHT).show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for msg in &self.chat_history {
                        ui.label(egui::RichText::new(msg).color(egui::Color32::BLACK));
                    }
                });      
             });

             panel.outer_margin = egui::style::Margin { left: PADDING_SIDE_OUTER, right: PADDING_SIDE_OUTER, top: PADDING_TOP_BOT_PANEL, bottom: PADDING_TOP_OUTER };
             egui::TopBottomPanel::bottom("panel2").frame(panel).min_height(INPUT_HEIGHT).show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized([ 695., 20. ], egui::TextEdit::singleline(&mut self.input));
                        if ui.button("Send").clicked() {
                            let to_console: String = format!("{} > {}", chrono::offset::Utc::now().format("%Y-%m-%d %H:%M"), self.input.clone());
                            self.chat_history.push(to_console.clone());
                            self.output = to_console + "\n";
                            self.input = String::new();
                        }
                    });
             });
        });

        self.talk();  

    }

    
    
}




