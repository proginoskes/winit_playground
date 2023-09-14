
use std::{
    fmt::Debug,
    time::Duration,
    fs::{self, File}, mem::{swap, replace}
};

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
    dpi::PhysicalSize
};
use pixels::{Pixels, SurfaceTexture};
use clap::Parser;

use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Eq, PartialEq)]
enum CellState {
    Live,
    Dead
}

#[derive(Copy, Clone)]
struct Cell {
    state : CellState,
    buffer: CellState
}

#[derive(Serialize, Deserialize, Debug)]
struct HabitatJSON {
   size : usize,
   rules: (Vec<usize>,Vec<usize>),
   cells : Vec<(usize, usize)>
}

struct Habitat {
    x : u32,
    y : u32,
    size: u32,
    idx: usize,
    birth: Vec<bool>,
    survival: Vec<bool>,
    cellmap : Vec<Cell>,
}

impl Habitat {
    fn is_in_habitat(&self, i : usize, wind_size : PhysicalSize<u32>) -> bool{
        let paint_x = (i%wind_size.width as usize) as u32;
        let paint_y = (i/wind_size.width as usize) as u32;

        return paint_x < self.x + self.size && paint_x >= self.x 
            && paint_y < self.y + self.size && paint_y >= self.y
    }
    fn run_life_round(& mut self){
        /*
         * 0 0 0
         * 0 i 0
         * 0 0 0
         */

        
        let is_on_end = self.idx % self.size as usize == self.size as usize-1;
        let is_on_start = self.idx % self.size as usize == 0;
        let is_at_top = self.idx < self.size as usize;
        let is_at_bottom = self.idx >= self.size.pow(2) as usize-self.size as usize;
         
        
        let live_counter = match is_on_end {
            true => 0,
            false => ( match is_at_top {
                true => 0,
                false => if self.cellmap[self.idx - self.size as usize + 1].state 
                    == CellState::Live {1}else{0}
            }) + (match is_at_bottom {
                true => 0,
                false => if self.cellmap[self.idx + self.size as usize + 1].state 
                    == CellState::Live {1}else{0} 
            }) + if self.cellmap[self.idx+1].state == CellState::Live {1}else{0}
        } + (match is_on_start {
            true => 0,
            false => (match is_at_top {
                true => 0,
                false => if self.cellmap[self.idx - self.size as usize - 1].state 
                    == CellState::Live {1}else{0}
            }) + (match is_at_bottom {
                true => 0,
                false => if self.cellmap[self.idx + self.size as usize - 1].state 
                    == CellState::Live {1}else{0} 
            }) + if self.cellmap[self.idx-1].state == CellState::Live{1}else{0}
        }) + (match is_at_top {
            true => 0,
            false => if self.cellmap[self.idx-self.size as usize].state 
                == CellState::Live{1}else{0}
        }) + (match is_at_bottom {
            true => 0,
            false => if self.cellmap[self.idx + self.size as usize].state
                == CellState::Live{1}else{0}
        });

        self.cellmap[self.idx].buffer = match self.cellmap[self.idx].state{
            CellState::Live=> match self.survival[live_counter]{
                true=>CellState::Live,
                false=>CellState::Dead
            },
            CellState::Dead=> match self.birth[live_counter]{
                true=>CellState::Live,
                false=>CellState::Dead
            }
        };       
        
    }
    fn flip_cells(&mut self) {
        self.cellmap = self.cellmap.iter_mut().map(|c| {
            c.state = c.buffer;
            *c
        }).collect();
    }
    fn next_cell(& mut self)->Cell{
        let a = self.cellmap[self.idx];
        self.run_life_round();
        self.idx = (self.idx + 1)%self.cellmap.len() as usize;
        a
    }
    fn new(
        x: u32, y: u32, size: u32, 
        cell_pairs : Vec<(usize,usize)>,
        birth_indices:Vec<usize>, survival_indices:Vec<usize>
    ) -> Self{
        let mut cmap = Vec::with_capacity((size*size) as usize);
        for _i in 0..(size*size) {
            cmap.push(Cell{state:CellState::Dead, buffer:CellState::Dead});
        }
 
        for c in cell_pairs {
            cmap[(size as usize*c.0)+c.1].state=CellState::Live;
        }

        let mut birth = Vec::new();
        birth.resize(9,false);
        let mut survival = Vec::new();
        survival.resize(9,false);
        
        for i in birth_indices {
            birth[i]=true;
        }
        for i in survival_indices {
            survival[i]=true;
        }

        Habitat{
            x,
            y,
            size,
            idx:0,
            cellmap:cmap.clone(), 
            birth,
            survival
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    file: String
}

fn main () {
    let args = Args::parse();

    let data = match fs::read_to_string(args.file) {
        Ok(s)=>s,
        Err(_)=>panic!("could not read file")
    };

    /*
    let data = r#"{
        "size":512,
        "birth":[3],
        "survival":[2,3,4],
        "cells":[[100,100],[99,101],[98,99],[98,100],[98,101]]
    }"#;
    */

    let hab_json : HabitatJSON = 
        match serde_json::from_str(&data){
            Ok(h)=>h,
            Err(d)=>{dbg!(d);panic!("could not deserialize")}
        };


    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let size = window.inner_size();

    let mut cell_locations : Vec<(usize,usize)> = Vec::new();
 
    for c in hab_json.cells {
        cell_locations.push(c);
    }

    let mut hab = Habitat::new(
        16, 16,
        hab_json.size as u32, 
        cell_locations, 
        hab_json.rules.0, 
        hab_json.rules.1
    );
    let surface_texture = SurfaceTexture::new(size.width,size.height,&window);
    let mut frame_buf = match Pixels::new(size.width,size.height, surface_texture) {
        Ok(res)=>res,
        Err(_)=>panic!("Could not create frame buffer!")
    };

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        control_flow.set_wait_timeout(Duration::from_millis(5));

        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::RedrawRequested(_) => {

                for (i, pixel) in frame_buf.frame_mut().chunks_exact_mut(4).enumerate(){
                    let rgba = if hab.is_in_habitat(i, window.inner_size()) {
                        match hab.next_cell().state {
                            CellState::Live => [0x58, 0xe8, 0x2b, 0xff],
                            CellState::Dead => [0x23, 0x23, 0x23, 0xff]
                        }
                    } else { 
                        [0xe8, 0x2b, 0x58, 0xff]
                    };
                    pixel.copy_from_slice(&rgba);
                }
                hab.flip_cells();
                if let Err(err) = frame_buf.render() {
                    println!("RenderError: {err}");
                    control_flow.set_exit();
                }
            },
            Event::WindowEvent{
                event: WindowEvent::Resized(size_re),
                ..
            } => {
                if let Err(err) = frame_buf.resize_buffer(size_re.width, size_re.height){
                    println!("ResizeError: {err}");
                    control_flow.set_exit();
                }
                if let Err(err) = frame_buf.resize_surface(size_re.width, size_re.height){
                    println!("ResizeError: {err}");
                    control_flow.set_exit();
                }
                window.request_redraw();
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                control_flow.set_exit();
            },
            _ => ()
        }
    });
}
