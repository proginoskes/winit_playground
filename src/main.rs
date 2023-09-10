
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
    dpi::PhysicalSize
};
use pixels::{Pixels, SurfaceTexture};

#[derive(Copy, Clone)]
enum CellState {
    Live,
    Dead
}

#[derive(Copy, Clone)]
struct Cell {
    state : CellState
}

struct Habitat {
    x : u32,
    y : u32,
    size: u32,
    idx: usize,
    cellmap : Vec<Cell>
}

impl Habitat {
    fn is_in_habitat(&self, i : usize, wind_size : PhysicalSize<u32>) -> bool{
        let paint_x = (i%wind_size.width as usize) as u32;
        let paint_y = (i/wind_size.width as usize) as u32;

        return paint_x < self.x + self.size && paint_x >= self.x 
            && paint_y < self.y + self.size && paint_y >= self.y
    }
    fn next_cell(& mut self)->Cell{
        let a = self.cellmap[self.idx];
        self.idx = (self.idx + 1)%self.size.pow(2) as usize;
        a
    }
    fn new(x: u32, y: u32, size: u32) -> Self{
        let mut cmap = Vec::with_capacity((size*size) as usize);
        for i in 0..(size*size) {
            cmap.push(Cell{state: if i % 255 != 0 {
                    CellState::Live
                } else {
                    CellState::Dead
                }
            });
        }
        Habitat{x,y,size,idx:0,cellmap:cmap}
    }
}



fn main () {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let size = window.inner_size();
    let mut hab = Habitat::new(16, 16, 512);
    let surface_texture = SurfaceTexture::new(size.width,size.height,&window);
    let mut frame_buf = match Pixels::new(size.width,size.height, surface_texture) {
        Ok(res)=>res,
        Err(_)=>panic!("Could not create frame buffer!")
    };

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        control_flow.set_wait();

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
