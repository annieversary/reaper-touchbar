use reaper_high::Reaper;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::{CommandId, ControlSurface, HookCommand};
use rubrail::ItemId;
use std::error::Error;

use rubrail::TTouchbar;
use rubrail::Touchbar;

static mut STATE: Option<State> = None;

#[reaper_extension_plugin(name = "reaper-touchbar", support_email_address = "annie@versary.town")]
fn plugin_main() -> Result<(), Box<dyn Error>> {
    let mut session = Reaper::get().medium_session();

    session
        .plugin_register_add_hook_command::<MyHookCommand>()
        .unwrap();
    session
        .plugin_register_add_csurf_inst(Box::new(MyControlSurface))
        .unwrap();

    // let reaper = session.reaper();

    let mut tb = Touchbar::alloc("reaper");
    let barid = tb.create_bar();
    let quit_id = tb.create_button(
        None,
        Some("bpm"),
        Box::new(move |_| {
            let r = Reaper::get().medium_reaper();
            let bpm = r.master_get_tempo().get();
            r.show_console_msg(format!("your bpm is set to {bpm}\n"));
        }),
    );

    let label1_id = tb.create_label("No last action");

    tb.add_items_to_bar(&barid, vec![quit_id, label1_id]);
    tb.set_bar_as_root(barid);

    // leak cause we need it to be up for the whole program
    unsafe {
        STATE = Some(State {
            tb: Box::leak(tb),
            label_id: label1_id,
        });
    }

    Ok(())
}

struct State {
    tb: &'static mut rubrail::touchbar::RustTouchbarDelegateWrapper,
    label_id: ItemId,
}

fn update_label(s: &str) {
    unsafe {
        let Some(state) = &mut STATE else {
            return;
        };
        state.tb.update_label(&state.label_id, s);
    }
}

struct MyHookCommand;

impl HookCommand for MyHookCommand {
    fn call(command_id: CommandId, _flag: i32) -> bool {
        // let r = Reaper::get().medium_reaper();
        // r.show_console_msg(format!("Executing command {command_id}!\n"));

        update_label(&format!("Last command: {command_id}"));

        false
    }
}

#[derive(Debug)]
struct MyControlSurface;

impl ControlSurface for MyControlSurface {
    fn on_track_selection(&self, args: reaper_medium::OnTrackSelectionArgs) {
        let r = Reaper::get().medium_reaper();
        unsafe {
            if let Ok(v) = r.get_track_ui_vol_pan(args.track) {
                update_label(&format!("Track volume: {}", v.volume.get()));
            }
        }
    }

    fn set_surface_volume(&self, args: reaper_medium::SetSurfaceVolumeArgs) {
        update_label(&format!("Track volume: {}", args.volume.get()));
    }
}
