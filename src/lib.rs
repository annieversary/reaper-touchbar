use reaper_high::Reaper;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::{CommandId, HookCommand};
use rubrail::ItemId;
use std::error::Error;

use rubrail::TTouchbar;
use rubrail::Touchbar;

static mut TB: Option<&'static mut rubrail::touchbar::RustTouchbarDelegateWrapper> = None;
static mut LABEL_ID: Option<ItemId> = None;

#[reaper_extension_plugin(name = "reaper-touchbar", support_email_address = "annie@versary.town")]
fn plugin_main() -> Result<(), Box<dyn Error>> {
    let mut session = Reaper::get().medium_session();

    session
        .plugin_register_add_hook_command::<MyHookCommand>()
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
        TB = Some(Box::leak(tb));
        LABEL_ID = Some(label1_id);
    }

    Ok(())
}

struct MyHookCommand;

impl HookCommand for MyHookCommand {
    fn call(command_id: CommandId, _flag: i32) -> bool {
        // let r = Reaper::get().medium_reaper();

        // TODO check if selecting a track is an action

        // r.show_console_msg(format!("Executing command {command_id}!\n"));

        unsafe {
            'tb: {
                let Some(tb) = &mut TB else {
                    break 'tb;
                };
                let Some(id) = LABEL_ID else {
                    break 'tb;
                };
                tb.update_label(&id, &format!("Last command: {command_id}"));
            }
        }
        false
    }
}
