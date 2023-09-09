use wayland_client::{
    event_created_child,
    protocol::{wl_output::Transform, wl_registry},
    Connection, Dispatch, EventQueue, QueueHandle, WEnum,
};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1::{self, ZwlrOutputConfigurationHeadV1},
    zwlr_output_configuration_v1::{self, ZwlrOutputConfigurationV1},
    zwlr_output_head_v1::{self, ZwlrOutputHeadV1},
    zwlr_output_manager_v1::{self, ZwlrOutputManagerV1},
    zwlr_output_mode_v1::{self, ZwlrOutputModeV1},
};

use crate::Orientation;

use super::AppLoop;

pub struct WaylandLoop {
    state: AppData,
    event_queue: EventQueue<AppData>,
}

impl WaylandLoop {
    pub fn new(conn: Connection, target_display: &str) -> WaylandLoop {
        let wl_display = conn.display();
        let mut event_queue = conn.new_event_queue();
        let _registry = wl_display.get_registry(&event_queue.handle(), ());
        let mut state = AppData::new(&mut event_queue, target_display.to_string());
        event_queue.roundtrip(&mut state).unwrap();
        // Roundtrip a second time to sync the outputs
        event_queue.roundtrip(&mut state).unwrap();
        // TODO: bail out if output management protocol isn't available
        WaylandLoop { state, event_queue }
    }
}
impl AppLoop for WaylandLoop {
    fn tick_always(&mut self) -> () {
        self.event_queue
            .roundtrip(&mut self.state)
            .expect("Failed to read display changes.");
    }
    fn tick(&mut self, new_state: &Orientation) {
        self.state.update_configuration(match new_state.new_state {
            "normal" => Transform::Normal,
            "90" => Transform::_270,
            "180" => Transform::_180,
            "270" => Transform::_90,
            &_ => Transform::Normal,
        });

        self.event_queue
            .flush()
            .expect("Failed to apply display changes.");
    }

    fn get_rotation_state(&self, display: &str) -> Result<String, String> {
        // TODO: implement
        return Ok("normal".to_string());
    }
}

struct AppData {
    target_display_name: String,
    target_head: Option<ZwlrOutputHeadV1>,
    output_manager: Option<ZwlrOutputManagerV1>,
    current_config_serial: Option<u32>,
    queue_handle: QueueHandle<AppData>,
}

// Public interface

impl AppData {
    pub fn new(event_queue: &mut EventQueue<AppData>, target_display_name: String) -> AppData {
        AppData {
            target_display_name,
            queue_handle: event_queue.handle(),
            target_head: None,
            output_manager: None,
            current_config_serial: None,
        }
    }

    pub fn update_configuration(&mut self, new_transform: Transform) {
        let output_manager = self
            .output_manager
            .as_ref()
            .expect("Failed to create wayland output manager.");
        // The serial should be replaced after applying the new config, so we can
        // avoid cloning here.
        let current_serial = match self.current_config_serial {
            Some(value) => value.clone(),
            None => return,
        };
        self.current_config_serial = Some(current_serial + 1u32);

        let target_head = self
            .target_head
            .as_ref()
            .expect("Failed to get target head.");
        let configuration =
            output_manager.create_configuration(current_serial, &self.queue_handle, ());
        let head_config = configuration.enable_head(&target_head, &self.queue_handle, ());
        head_config.set_transform(new_transform);
        configuration.apply();
    }
}

// Event handlers

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        _state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{}] {} v{}", name, interface, version);
            if interface == "zwlr_output_manager_v1" {
                _state.output_manager =
                    Some(registry.bind::<ZwlrOutputManagerV1, (), AppData>(name, version, qh, ()));
            }
        }
    }
}

impl Dispatch<ZwlrOutputManagerV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _: &ZwlrOutputManagerV1,
        event: zwlr_output_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        if let zwlr_output_manager_v1::Event::Done { serial } = event {
            println!("Current config: {}", serial);
            _state.current_config_serial = Some(serial);
        }
    }

    event_created_child!(AppData, ZwlrOutputHeadV1, [
       zwlr_output_manager_v1::EVT_HEAD_OPCODE => (ZwlrOutputHeadV1, ()),
    ]);
}

impl Dispatch<ZwlrOutputHeadV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        head: &ZwlrOutputHeadV1,
        event: zwlr_output_head_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        match event {
            zwlr_output_head_v1::Event::Name { name } => {
                if name == _state.target_display_name {
                    println!("Found target display: {}", name);
                    _state.target_head = Some(head.clone());
                }
            }
            zwlr_output_head_v1::Event::Transform { transform } => {
                println!(
                    "New transform: {}",
                    match transform {
                        WEnum::Value(Transform::_90) => "90",
                        WEnum::Value(Transform::_180) => "180",
                        WEnum::Value(Transform::_270) => "270",
                        _ => "normal",
                    }
                );
            }
            _ => {}
        }
    }

    event_created_child!(AppData, ZwlrOutputModeV1, [
       zwlr_output_head_v1::EVT_CURRENT_MODE_OPCODE => (ZwlrOutputModeV1, ()),
       zwlr_output_head_v1::EVT_MODE_OPCODE => (ZwlrOutputModeV1, ()),
    ]);
}

impl Dispatch<ZwlrOutputModeV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _: &ZwlrOutputModeV1,
        _: zwlr_output_mode_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
    }
}

impl Dispatch<ZwlrOutputConfigurationV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        config: &ZwlrOutputConfigurationV1,
        event: zwlr_output_configuration_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
        match event {
            zwlr_output_configuration_v1::Event::Succeeded => {
                println!("Config applied successfully.");
                config.destroy();
            }
            zwlr_output_configuration_v1::Event::Failed => {
                println!("Failed to apply new config.");
                config.destroy();
            }
            zwlr_output_configuration_v1::Event::Cancelled => {
                println!("Config application cancelled.");
                config.destroy();
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwlrOutputConfigurationHeadV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _: &ZwlrOutputConfigurationHeadV1,
        _: zwlr_output_configuration_head_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<AppData>,
    ) {
    }
}
