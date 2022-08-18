// #[macro_use] asks the complier to import the macros defined in the `events`
// module. This is necessary because macros cannot be namespaced -- macro 
// expansion happens before the concept of namespace event starts to _exist_ in
// the compilation timeline.
#[macro_use]
mod events;

use sdl2::render::WindowCanvas;

struct_events! {
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down
    },
    else: {
        quit: Quit { .. }
    }
}

/// Bundles the Phi abstractions in a single structure witch
/// can be passed easily between functions.
pub struct Phi {
    pub events: Events,
    pub renderer: WindowCanvas,
}

/// A `ViewAction` is a way for the currently executed view to
/// communicate with the game loop. It specifies which action
/// should be executed before the next rendering.
pub enum ViewAction {
    None,
    Quit,
}

pub trait View {
    /// Called on every frame to take care of both the logic and
    /// the rendering of the current view
    /// 
    /// `elapsed` is expressed in seconds.
    fn render(&mut self, context: &mut Phi, elapsed: f64) -> ViewAction;
}

pub fn spawn<F>(title: &str, init: F)
where 
    F: Fn(&mut Phi) -> Box<dyn View>
{
    // Initialize sdl2
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut timer = sdl_context.timer().unwrap();

    // Create the window
    let window = video.window("ArcadeRS Shooter", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    // Create the context
    let mut context = Phi {
        events: Events::new(sdl_context.event_pump().unwrap()),
        renderer: window.into_canvas()
            .accelerated()
            .build().unwrap(),
    };
    
    // Create the default view
    let mut current_view: Box<dyn View> = Box::new(crate::views::DefaultView);

    // Frame timing
    let interval = 1_000 / 60;
    let mut before = timer.ticks();
    let mut last_second = timer.ticks();
    let mut fps = 0u16;

    loop {
        // Frame timing (bis)

        let now = timer.ticks();
        let dt = now - before;
        let elapsed = dt as f64 / 1_000.0;

        // If the time elapsed since the last frame is too small, wait out the
        // difference and try again.
        if dt < interval {
            timer.delay(interval - dt);
            continue;
        }

        before = now;
        fps += 1;

        if now - last_second > 1_000 {
            println!("FPS: {}", fps);
            last_second = now;
            fps = 0;
        }


        // Logic & rendering

        context.events.pump();

        match current_view.render(&mut context, 0.01) {
            ViewAction::None => context.renderer.present(),
            ViewAction::Quit => break,
        }
    }
}