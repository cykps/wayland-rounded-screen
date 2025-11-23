mod corner;
use std::{convert::TryInto, num::NonZeroU32};

use corner::{CornerLayer, RADIUS, create_layer_surface};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, Region},
    delegate_compositor,
    delegate_layer,
    delegate_output,
    delegate_registry,
    delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    // seat::input_method_v3::Anchor,
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
    },
    shm::{Shm, ShmHandler, slot::SlotPool},
};
use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
};

fn main() {
    env_logger::init();

    // All Wayland apps start by connecting the compositor (server).
    let conn = Connection::connect_to_env().unwrap();

    // Enumerate the list of globals to get the protocols the server implements.
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    // The compositor (not to be confused with the server which is commonly called the compositor) allows
    // configuring surfaces to be presented.
    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
    // This app uses the wlr layer shell, which may not be available with every compositor.
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");

    let mut layers = vec![
        create_layer_surface(
            &compositor,
            &globals,
            &qh,
            &layer_shell,
            Anchor::TOP | Anchor::LEFT,
        ),
        create_layer_surface(
            &compositor,
            &globals,
            &qh,
            &layer_shell,
            Anchor::TOP | Anchor::RIGHT,
        ),
        create_layer_surface(
            &compositor,
            &globals,
            &qh,
            &layer_shell,
            Anchor::BOTTOM | Anchor::LEFT,
        ),
        create_layer_surface(
            &compositor,
            &globals,
            &qh,
            &layer_shell,
            Anchor::BOTTOM | Anchor::RIGHT,
        ),
    ];

    // We don't draw immediately, the configure will notify us when to first draw.
    loop {
        event_queue.blocking_dispatch(&mut layers[0]).unwrap();
        //
        if layers[0].exit {
            println!("exiting example");
            break;
        }
    }
}
