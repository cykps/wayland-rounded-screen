mod state;
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

pub struct CornerState {
    first_configure: bool,
    layer: LayerSurface,
    pool: SlotPool,
}

// Implemations

impl CornerState {
    fn new(
        layer_shell: &LayerShell,
        compositor: &CompositorState,
        shm: &Shm,
        qh: &QueueHandle<State>,
        radius: u32,
        anchor: Anchor,
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

        layer.set_anchor(anchor);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(RADIUS, RADIUS);
        layer.set_exclusive_zone(-1);
        layer.commit();

        let pool = SlotPool::new(256 * 256 * 4, shm).expect("Failed to create pool");

        CornerState {
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
                    let x = (index % width as usize) as u32;
                    let y = (index / width as usize) as u32;
                    let transport = 0x00000000;
                    let black = 0xFF000000;

                    let color: u32 = if x.pow(2) + y.pow(2) <= width.pow(2) {
                        // transport // TODO
                        black
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
