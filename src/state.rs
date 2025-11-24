mod consts;
use consts::RADIUS;

use std::{convert::TryInto, num::NonZeroU32};

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, Region},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
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
    globals::GlobalList,
    protocol::{wl_output, wl_shm, wl_surface},
};

// Structs

pub struct State {
    top_left: CornerState,
    top_right: CornerState,
    bottom_left: CornerState,
    bottom_right: CornerState,

    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    pub exit: bool,
    radius: u32,
}

// Implemations
impl State {
    pub fn new(
        compositor: &CompositorState,
        globals: &GlobalList,
        qh: &QueueHandle<State>,
        layer_shell: &LayerShell,
    ) -> State {
        let shm = Shm::bind(globals, qh).expect("wl_shm is not available");

        let top_left = CornerState::new(
            layer_shell,
            compositor,
            &shm,
            qh,
            Anchor::TOP | Anchor::LEFT,
        );
        let top_right = CornerState::new(
            layer_shell,
            compositor,
            &shm,
            qh,
            Anchor::TOP | Anchor::RIGHT,
        );
        let bottom_left = CornerState::new(
            layer_shell,
            compositor,
            &shm,
            qh,
            Anchor::BOTTOM | Anchor::LEFT,
        );
        let bottom_right = CornerState::new(
            layer_shell,
            compositor,
            &shm,
            qh,
            Anchor::BOTTOM | Anchor::RIGHT,
        );

        State {
            top_left,
            top_right,
            bottom_left,
            bottom_right,

            registry_state: RegistryState::new(globals),
            output_state: OutputState::new(globals, qh),
            shm,

            exit: false,
            radius: RADIUS,
        }
    }
}

impl ProvidesRegistryState for State {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

impl ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_compositor!(State);
delegate_output!(State);
delegate_shm!(State);
delegate_layer!(State);
delegate_registry!(State);
