use reaper_high::Guid;
use reaper_high::Reaper;
use reaper_high::Track;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::{CommandId, ControlSurface, HookCommand};
use rubrail::BarId;
use rubrail::ItemId;
use std::collections::HashSet;
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

    let mut tb = Touchbar::alloc("Reaper");
    let bar_id = tb.create_bar();

    let bpm_id = tb.create_button(
        None,
        Some("Log bpm"),
        Box::new(move |_| {
            let r = Reaper::get().medium_reaper();
            let bpm = r.master_get_tempo().get();
            r.show_console_msg(format!("your bpm is set to {bpm}\n"));
        }),
    );
    let label_id = tb.create_label("meow");
    let mute_button = tb.create_button(None, Some("Mute"), Box::new(mute_selected));
    let solo_button = tb.create_button(None, Some("Solo"), Box::new(solo_selected));

    tb.add_items_to_bar(&bar_id, vec![label_id, bpm_id]);
    tb.set_bar_as_root(bar_id);

    unsafe {
        STATE = Some(State {
            // TODO maybe we don't need to leak it this
            tb: Box::leak(tb),
            label_id,
            bar_id,

            normal_tb_elements: vec![label_id, bpm_id],
            track_tb_elements: vec![label_id, mute_button, solo_button],

            selected_tracks: HashSet::new(),
            mode: Mode::Normal,
        });
    }

    Ok(())
}

struct State {
    tb: &'static mut rubrail::touchbar::RustTouchbarDelegateWrapper,
    label_id: ItemId,
    bar_id: BarId,

    normal_tb_elements: Vec<ItemId>,
    track_tb_elements: Vec<ItemId>,

    selected_tracks: HashSet<Guid>,
    mode: Mode,
}

impl State {
    fn change_mode(&mut self, new_mode: Mode) {
        if new_mode != self.mode {
            self.mode = new_mode;

            match self.mode {
                Mode::Normal => {
                    self.tb
                        .add_items_to_bar(&self.bar_id, self.normal_tb_elements.clone());
                }
                Mode::Track => {
                    self.tb
                        .add_items_to_bar(&self.bar_id, self.track_tb_elements.clone());
                }
            }
        }
    }
}

#[derive(PartialEq, Eq)]
enum Mode {
    Normal,
    Track,
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
    fn call(_command_id: CommandId, _flag: i32) -> bool {
        // let r = Reaper::get().medium_reaper();
        // r.show_console_msg(format!("Executing command {command_id}!\n"));

        // update_label(&format!("Last command: {command_id}"));

        false
    }
}

#[derive(Debug)]
struct MyControlSurface;

impl ControlSurface for MyControlSurface {
    fn set_surface_selected(&self, args: reaper_medium::SetSurfaceSelectedArgs) {
        unsafe {
            let Some(state) = &mut STATE else {
                return;
            };

            let track = Track::new(args.track, None);
            let id = track.guid();

            if args.is_selected {
                state.selected_tracks.insert(*id);
            } else {
                state.selected_tracks.remove(id);
            }

            if state.selected_tracks.is_empty() {
                state.change_mode(Mode::Normal);
                update_label("No tracks selected")
            } else {
                state.change_mode(Mode::Track);
                update_label(&format!("{} tracks selected", state.selected_tracks.len()))
            }
        }
    }

    // fn set_surface_volume(&self, args: reaper_medium::SetSurfaceVolumeArgs) {
    //     update_label(&format!("Track volume: {}", args.volume.get()));
    // }
}

fn mute_selected(_: &u64) {
    unsafe {
        let Some(state) = &mut STATE else {
            return;
        };

        let reaper = Reaper::get();
        let project = reaper.current_project();

        for track_guid in &state.selected_tracks {
            if let Ok(track) = project.track_by_guid(track_guid) {
                if track.is_muted() {
                    track.unmute(
                        reaper_medium::GangBehavior::AllowGang,
                        reaper_high::GroupingBehavior::UseGrouping,
                    );
                } else {
                    track.mute(
                        reaper_medium::GangBehavior::AllowGang,
                        reaper_high::GroupingBehavior::UseGrouping,
                    );
                }
            }
        }
    }
}
fn solo_selected(_: &u64) {
    unsafe {
        let Some(state) = &mut STATE else {
            return;
        };

        let reaper = Reaper::get();
        let project = reaper.current_project();

        for track_guid in &state.selected_tracks {
            if let Ok(track) = project.track_by_guid(track_guid) {
                if track.is_solo() {
                    track.unsolo(
                        reaper_medium::GangBehavior::AllowGang,
                        reaper_high::GroupingBehavior::UseGrouping,
                    );
                } else {
                    track.solo(
                        reaper_medium::GangBehavior::AllowGang,
                        reaper_high::GroupingBehavior::UseGrouping,
                    );
                }
            }
        }
    }
}
