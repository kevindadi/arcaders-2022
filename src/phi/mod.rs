// #[macro_use] asks the complier to import the macros defined in the `events`
// module. This is necessary because macros cannot be namespaced -- macro 
// expansion happens before the concept of namespace event starts to _exist_ in
// the compilation timeline.
#[macro_use]
mod events;
pub mod data;
pub mod gfx;

use sdl2::render::WindowCanvas;
use self::gfx::Sprite;
use sdl2::pixels::Color;
use std::path::Path;

struct_events! {
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down,
        key_left: Left,
        key_right: Right,
        key_space: Space,
        key_enter: Return,

        key_1: Num1,
        key_2: Num2,
        key_3: Num3
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

impl Phi{
    fn new(events: Events, renderer: WindowCanvas) -> Phi {
        Phi {
            events: events,
            renderer: renderer,
        }
    }

    pub fn output_size(&self) -> (f64, f64) {
        let (w, h) = self.renderer.output_size().unwrap();
        (w as f64, h as f64)
    }

    pub fn ttf_str_sprite(&mut self, text: &str, font_path: &'static str, size: i32, color: Color) -> Option<Sprite> {
        ::sdl2::ttf::init().unwrap().load_font(Path::new(font_path), size as u16).ok()
            .and_then(|font| font
                .render(text).blended(color).ok()
                .and_then(|surface| self.renderer.create_texture_from_surface(&surface).ok())
                .map(Sprite::new)
        )
    }
}

/// A `ViewAction` is a way for the currently executed view to
/// communicate with the game loop. It specifies which action
/// should be executed before the next rendering.
pub enum ViewAction {
    None,
    Quit,
    ChangeView(Box<dyn View>),
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
    let _image_context = ::sdl2::image::init(::sdl2::image::InitFlag::PNG).unwrap();
    
    // Initialize audio plugin
    //? We will stick to the Ogg format throughout this article. However, you
    //? can easily require other ones.
    // let _mixer_context = ::sdl2::mixer::init(::sdl2::mixer::InitFlag::OGG).unwrap();
    //? We configure our audio context so that:
    //?   * The frequency is 44100;
    //?   * Use signed 16 bits samples, in little-endian byte order;
    //?   * It's also stereo (2 "channels");
    //?   * Samples are 1024 bytes in size.
    //? You don't really need to understand what all of this means. I myself just
    //? copy-pasted this from andelf's demo. ;-)
    ::sdl2::mixer::open_audio(44100, ::sdl2::mixer::AUDIO_S16LSB, 2, 1024).unwrap();
    //? This function asks us how many channels we wish to allocate for our game.
    //? That is, how many sounds do we wish to be able to play at the same time?
    //? While testing, 16 channels seemed to be sufficient. Which means that we
    //? should probably request 32 of 'em just in case. :-°
    ::sdl2::mixer::allocate_channels(32);

    // Create the window
    let window = video.window(title, 800, 600)
        .position_centered()
        .opengl()
        .resizable()
        .build()
        .unwrap();

    // Create the context
    let mut context = Phi::new(
        Events::new(sdl_context.event_pump().unwrap()),
        window.into_canvas()
            .accelerated()
            .build().unwrap(),
    );
    
    // Create the default view
    let mut current_view = init(&mut context);

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

        context.events.pump(&mut context.renderer);

        match current_view.render(&mut context, elapsed) {
            ViewAction::None => context.renderer.present(),
            ViewAction::Quit => break,
            ViewAction::ChangeView(new_view) => current_view = new_view,
        }
    }
}