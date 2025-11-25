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

pub const RADIUS: u32 = 24;
// pub const RADIUS: u32 = 256;

// enum
enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

// CornerState Struct
pub struct CornerState {
    disposition: Quadrant,
    first_configure: bool,
    layer: LayerSurface,
    pool: SlotPool,
}

impl CornerState {
    fn new(
        layer_shell: &LayerShell,
        compositor: &CompositorState,
        shm: &Shm,
        qh: &QueueHandle<State>,
        disposition: Quadrant,
    ) -> CornerState {
        let surface = compositor.create_surface(qh);
        let region = Region::new(compositor).expect("region is not available");
        surface.set_input_region(Some(region.wl_region()));

        let layer = layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Overlay,
            Some("corner_layer"),
            None,
        );

        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(RADIUS, RADIUS);
        layer.set_exclusive_zone(-1);

        let anchor = match disposition {
            Quadrant::TopLeft => Anchor::TOP | Anchor::LEFT,
            Quadrant::TopRight => Anchor::TOP | Anchor::RIGHT,
            Quadrant::BottomLeft => Anchor::BOTTOM | Anchor::LEFT,
            Quadrant::BottomRight => Anchor::BOTTOM | Anchor::RIGHT,
        };
        layer.set_anchor(anchor);

        layer.commit();

        let pool = SlotPool::new(256 * 256 * 4, shm).expect("Failed to create pool");

        CornerState {
            disposition,
            first_configure: true,
            layer,
            pool,
        }
    }

    fn configure(&mut self, qh: &QueueHandle<State>) {
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }

    pub fn draw(&mut self, _qh: &QueueHandle<State>) {
        let width = RADIUS;
        let height = RADIUS;
        let stride = RADIUS as i32 * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");

        // Draw to the window:
        {
            canvas
                .chunks_exact_mut(4)
                .enumerate()
                .for_each(|(index, chunk)| {
                    let x = match self.disposition {
                        Quadrant::TopLeft | Quadrant::BottomLeft => {
                            width - (index % width as usize) as u32
                        }
                        Quadrant::TopRight | Quadrant::BottomRight => {
                            (index % width as usize) as u32
                        }
                    };
                    let y = match self.disposition {
                        Quadrant::TopLeft | Quadrant::TopRight => {
                            height - (index / width as usize) as u32
                        }
                        Quadrant::BottomRight | Quadrant::BottomLeft => {
                            (index / width as usize) as u32
                        }
                    };
                    let transport = 0x00000000;
                    let black = 0xFF000000;

                    let color: u32 = if x.pow(2) + y.pow(2) <= width.pow(2) {
                        transport
                    } else {
                        black
                    };

                    let array: &mut [u8; 4] = chunk.try_into().unwrap();
                    *array = color.to_le_bytes()
                });
        }

        // Damage the entire window
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // Attach and commit to present.
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();
    }
}

// State Structs
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

impl CompositorHandler for State {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for State {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for State {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.radius = NonZeroU32::new(configure.new_size.1).map_or(RADIUS, NonZeroU32::get);

        if *layer == self.top_left.layer {
            self.top_left.configure(qh);
        } else if *layer == self.top_right.layer {
            self.top_right.configure(qh);
        } else if *layer == self.bottom_left.layer {
            self.bottom_left.configure(qh);
        } else if *layer == self.bottom_right.layer {
            self.bottom_right.configure(qh);
        }
    }
}

impl ShmHandler for State {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl State {
    pub fn new(
        compositor: &CompositorState,
        globals: &GlobalList,
        qh: &QueueHandle<State>,
        layer_shell: &LayerShell,
    ) -> State {
        let shm = Shm::bind(globals, qh).expect("wl_shm is not available");

        let top_left = CornerState::new(layer_shell, compositor, &shm, qh, Quadrant::TopLeft);
        let top_right = CornerState::new(layer_shell, compositor, &shm, qh, Quadrant::TopRight);
        let bottom_left = CornerState::new(layer_shell, compositor, &shm, qh, Quadrant::BottomLeft);
        let bottom_right =
            CornerState::new(layer_shell, compositor, &shm, qh, Quadrant::BottomRight);

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

delegate_compositor!(State);
delegate_output!(State);
delegate_shm!(State);
delegate_layer!(State);
delegate_registry!(State);
