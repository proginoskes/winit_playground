
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
    //platform::x11::WindowExtX11
};
use pixels::{Pixels, SurfaceTexture};

fn main () {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let size = window.inner_size();
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

                for (_i, pixel) in frame_buf.frame_mut().chunks_exact_mut(4).enumerate(){
                    let rgba = [0x48, 0xb2, 0x8e, 0xff];
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
                if let Err(err) = frame_buf.resize_surface(size_re.width, size_re.height){
                    println!("ResizeError: {err}");
                    control_flow.set_exit();
                }
                if let Err(err) = frame_buf.resize_buffer(size_re.width, size_re.height){
                    println!("ResizeError: {err}");
                    control_flow.set_exit();
                }
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
