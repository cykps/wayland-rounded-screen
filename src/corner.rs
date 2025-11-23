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
    globals::{GlobalList, registry_queue_init},
    protocol::{wl_output, wl_shm, wl_surface},
};

pub const RADIUS: u32 = 24;

pub fn create_layer_surface(
    compositor: &CompositorState,
    globals: &GlobalList,
    qh: &QueueHandle<CornerLayer>,
    layer_shell: &LayerShell,
    anchor: Anchor,
) -> CornerLayer {
    // Since we are not using the GPU in this example, we use wl_shm to allow software rendering to a buffer
    // we share with the compositor process.
    let shm = Shm::bind(&globals, qh).expect("wl_shm is not available");
    // A layer surface is created from a surface.
    let surface = compositor.create_surface(qh);

    let region = Region::new(compositor).expect("region is not available");
    surface.set_input_region(Some(region.wl_region()));

    // And then we create the layer shell.
    let layer =
        layer_shell.create_layer_surface(qh, surface, Layer::Overlay, Some("simple_layer"), None);
    // Configure the layer surface, providing things like the anchor on screen, desired size and the keyboard
    // interactivity
    layer.set_anchor(anchor);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.set_size(RADIUS, RADIUS);
    layer.set_exclusive_zone(-1);

    // In order for the layer surface to be mapped, we need to perform an initial commit with no attached\
    // buffer. For more info, see WaylandSurface::commit
    //
    // The compositor will respond with an initial configure that we can then use to present to the layer
    // surface with the correct options.
    layer.commit();

    // We don't know how large the window will be yet, so lets assume the minimum size we suggested for the
    // initial memory allocation.
    let pool = SlotPool::new(RADIUS.pow(2) as usize * 4, &shm).expect("Failed to create pool");

    let corner_layer = CornerLayer {
        // Seats and outputs may be hotplugged at runtime, therefore we need to setup a registry state to
        // listen for seats and outputs.
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,

        exit: false,
        first_configure: true,
        pool,
        radius: RADIUS,
        layer,
    };

    return corner_layer;
}

// pub struct CornerLayers {
//     layers: Vec<CornerLayer>,
// }

pub struct CornerLayer {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,

    exit: bool,
    first_configure: bool,
    pool: SlotPool,
    radius: u32,
    layer: LayerSurface,
}

impl CompositorHandler for CornerLayer {
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

impl OutputHandler for CornerLayer {
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

impl LayerShellHandler for CornerLayer {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.radius = NonZeroU32::new(configure.new_size.1).map_or(RADIUS, NonZeroU32::get);

        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl ShmHandler for CornerLayer {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl CornerLayer {
    pub fn draw(&mut self, _qh: &QueueHandle<Self>) {
        let width = self.radius;
        let height = self.radius;
        let stride = self.radius as i32 * 4;

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
                    let x = (index % width as usize) as u32;
                    let y = (index / width as usize) as u32;
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

        // Request our next frame
        // self.layer
        //     .wl_surface()
        //     .frame(qh, self.layer.wl_surface().clone());

        // Attach and commit to present.
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();

        // TODO save and reuse buffer when the window size is unchanged.  This is especially
        // useful if you do damage tracking, since you don't need to redraw the undamaged parts
        // of the canvas.
    }
}

impl ProvidesRegistryState for CornerLayer {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_compositor!(CornerLayer);
delegate_output!(CornerLayer);
delegate_shm!(CornerLayer);
delegate_layer!(CornerLayer);
delegate_registry!(CornerLayer);
