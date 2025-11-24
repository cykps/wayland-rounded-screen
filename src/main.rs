mod corner;

use corner::State;
use smithay_client_toolkit::{compositor::CompositorState, shell::wlr_layer::LayerShell};
use wayland_client::{Connection, globals::registry_queue_init};

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();

    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");

    let mut state = State::new(&compositor, &globals, &qh, &layer_shell);

    // We don't draw immediately, the configure will notify us when to first draw.
    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
        if state.exit {
            println!("exiting");
            break;
        }
    }
}
