//! wlr-layer-shell support (AGENTS section 6.7: panel and optional dock).
//!
//! Layer surfaces (the ArchLume panel, and later the dock) are Wayland clients
//! that ask to be placed on the top/bottom/overlay/background layers. Smithay
//! keeps them in a per-output `LayerMap`; `Space::render_output` renders the
//! maps automatically, so this module only has to map new surfaces, unmap them
//! on destruction, and send the initial configure after the client's first
//! commit (the protocol requires the initial configure to follow that commit).

use crate::ArchLume;
use smithay::{
    delegate_layer_shell,
    desktop::{layer_map_for_output, LayerSurface},
    output::Output,
    reexports::wayland_server::protocol::wl_output::WlOutput,
    wayland::shell::wlr_layer::{
        Layer, LayerSurface as WlrLayerSurface, WlrLayerShellHandler, WlrLayerShellState,
    },
};

impl WlrLayerShellHandler for ArchLume {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: WlrLayerSurface,
        wl_output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        // Fall back to the first output when the client did not name one.
        let output = wl_output
            .as_ref()
            .and_then(Output::from_resource)
            .or_else(|| self.output.clone());

        if let Some(output) = output {
            let mut map = layer_map_for_output(&output);
            if let Err(err) = map.map_layer(&LayerSurface::new(surface, namespace)) {
                eprintln!("failed to map layer surface: {err}");
            }
        }
    }

    fn layer_destroyed(&mut self, surface: WlrLayerSurface) {
        if let Some((mut map, layer)) = self.space.outputs().find_map(|output| {
            let map = layer_map_for_output(output);
            let layer = map
                .layers()
                .find(|layer| layer.layer_surface() == &surface)
                .cloned();
            layer.map(|layer| (map, layer))
        }) {
            map.unmap_layer(&layer);
        }
    }
}

delegate_layer_shell!(ArchLume);
