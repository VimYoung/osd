use slint::ComponentHandle;
use spell_framework::{
    cast_spell,
    layer_properties::{BoardType, DataType, ForeignController, LayerAnchor, WindowConf},
    wayland_adapter::SpellWin,
};
use std::{
    error::Error,
    sync::{Arc, RwLock},
};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let window_conf = WindowConf::new(
        80,
        250,
        (Some(LayerAnchor::RIGHT), None),
        (0, 0, 0, 0),
        spell_framework::layer_properties::LayerType::Top,
        BoardType::None,
        false,
    );
    let mut waywin = SpellWin::invoke_spell("vol-osd", window_conf);
    let ui = VolumeOSD::new().unwrap();
    let ui_clone = ui.as_weak().clone();
    let state = Box::new(ui.get_state());
    let handle = waywin.get_handler();
    handle.subtract_input_region(0, 0, 80, 248);
    ui.on_call_close({
        let h_clone = handle.clone();
        move || {
            h_clone.subtract_input_region(0, 0, 80, 250);
        }
    });

    ui.on_get_vol_value({
        let ui_handle = ui.as_weak().unwrap();
        let h_clone = handle.clone();
        move || {
            h_clone.add_input_region(0, 0, 80, 250);
            let comm = String::from("pamixer --get-volume");
            let mut out_vec = std::process::Command::new("sh")
                .arg("-c")
                .arg(comm)
                .output()
                .unwrap()
                .stdout;
            out_vec.pop();
            let val = format!("{}{}", String::from_utf8(out_vec).unwrap(), ".0");
            let changed_value = val.parse::<f32>().unwrap();
            ui_handle.set_value(changed_value);
            // changed_value
        }
    });

    ui.on_vol_changed(|vol| {
        let val = (vol as u32).to_string();
        let comm = String::from("pamixer --set-volume ") + &val;
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(comm)
            .output()
            .unwrap();
    });

    ui.on_toggle_mute(|| {
        let comm = String::from("pamixer -t");
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(comm)
            .output()
            .unwrap();
    });

    cast_spell(
        waywin,
        Some(Arc::new(RwLock::new(state))),
        Some(Box::new(
            move |state_value: Arc<RwLock<Box<dyn ForeignController>>>| {
                let controller_val = state_value.read().unwrap();
                let inner = controller_val.as_ref();
                let val = inner.as_any().downcast_ref::<osdState>().unwrap().clone();
                ui_clone.unwrap().set_state(val);
            },
        )),
    )
}

impl ForeignController for osdState {
    fn get_type(&self, key: &str) -> spell_framework::layer_properties::DataType {
        match key {
            "is-open" => DataType::Boolean(self.is_open),
            "is-restart" => DataType::Boolean(self.restart),
            // "val" => DataType::Int(self.value as i32),
            _ => DataType::Panic,
        }
    }

    fn change_val(&mut self, _: &str, val: DataType) {
        if let DataType::Boolean(some_val) = val
            && self.is_open != some_val
        {
            self.is_open = some_val;
        }
        if self.is_open {
            self.restart = true;
        }
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
