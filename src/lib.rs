#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder
};

mod state;

/* run as standalone application */
#[cfg(target_arch = "any_not_wasm")]
fn main() {
    // run
    pollster::block_on(run());
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn wasm_main() {
    // run
    let future = async {
        run().await;
    };
    wasm_bindgen_futures::spawn_local(future);
}

pub async fn run() {
    // set up logging
    #[cfg(target_arch = "any_not_wasm")]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    }

    // create window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // web-specific logic
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys; // ide doesnt realise platform is wasm; #[cfg(wasm_platform)]
        use winit::dpi::PhysicalSize;

        // attach winit window to html canvas
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("webassembly")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                // this is somehow scaling the canvas to exact right size on refresh, but is not flexible after that; why is 1080 not too big???
                canvas.set_attribute("style", "width: 1920px; height: 1080px").unwrap();
                Some(())
            })
            .expect("Couldn't append canvas to document body.");

        // auto resize the window; uses unsafe pointer to window/state:
        let window_ptr: *mut winit::window::Window = &window as *const _ as *mut _;

        let resize_closure = Closure::wrap(Box::new(move || {

            // get the size of the browser tab in CSS-PIXELS
            let js_window = web_sys::window().expect("should have a window in this context");
            let width = js_window.inner_width().unwrap().as_f64().unwrap() as f32;
            let height = js_window.inner_height().unwrap().as_f64().unwrap() as f32;
            // ratio of CSS-PIXELS to SCREEN PIXELS
            let (css_ratio_x, css_ratio_y) = (1536.0 / 1920.0, 864.0 / 1080.0);
            // subtract 15 regular pixels to accommodate html border and padding
            let new_size = PhysicalSize::new((width / css_ratio_x) - 15.0, (height / css_ratio_y) - 15.0);
            let window = unsafe { &mut *window_ptr };
            // set window size in SCREEN PIXELS
            window.set_inner_size(new_size);
            // log the size i just set
            log::info!("setting window with size: {:?}", new_size);

        }) as Box<dyn FnMut()>);

        web_sys::window()
            .expect("should have a window in this context")
            .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
            .expect("Failed to add resize event listener");

        resize_closure.forget();
    }

    // wgpu
    let state = state::State::new(window).await;

    // run event loop
    run_event_loop(event_loop, state);

}

fn run_event_loop(event_loop: EventLoop<()>, mut state: state::State) {
    // start window event loop
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { ref event, window_id } if window_id == state.window().id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == state.window.id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            state.window().request_redraw();
        },
        _ => {}
    });
}
