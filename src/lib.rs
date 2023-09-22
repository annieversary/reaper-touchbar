use reaper_high::Guid;
use reaper_high::Reaper;
use reaper_high::Track;
use reaper_high::Volume;
use reaper_macros::reaper_extension_plugin;
use reaper_medium::Db;
use reaper_medium::MasterTrackBehavior;
use reaper_medium::{CommandId, ControlSurface, HookCommand};
use rubrail::BarId;
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

    let mut tb = Touchbar::alloc("Reaper");
    let bar_id = tb.create_bar();

    let label_id = tb.create_label("meow");

    // normal items
    let bpm_id = tb.create_button(
        None,
        Some("Log bpm"),
        Box::new(move |_| {
            let r = Reaper::get().medium_reaper();
            let bpm = r.master_get_tempo().get();
            r.show_console_msg(format!("your bpm is set to {bpm}\n"));
        }),
    );

    // track items
    let mute_button = tb.create_button(None, Some("Mute"), Box::new(mute_selected));
    let solo_button = tb.create_button(None, Some("Solo"), Box::new(solo_selected));
    let volume_slider = tb.create_slider(0.0, 1.0, Some("Volume"), true, Box::new(volume_selected));

    tb.add_items_to_bar(&bar_id, vec![label_id]);
    tb.set_bar_as_root(bar_id);

    let project = Reaper::get().current_project();
    let last_selected_track = project
        .first_selected_track(reaper_medium::MasterTrackBehavior::ExcludeMasterTrack)
        .map(|t| *t.guid());

    unsafe {
        STATE = Some(State {
            // TODO maybe we don't need to leak it this
            tb: Box::leak(tb),
            label_id,
            bar_id,
            track_volume_id: volume_slider,

            normal_tb_elements: vec![label_id, bpm_id],
            track_tb_elements: vec![label_id, mute_button, solo_button, volume_slider],

            last_selected_track,
            mode: Mode::Normal,
        });
    }

    Ok(())
}

struct State {
    tb: &'static mut rubrail::touchbar::RustTouchbarDelegateWrapper,
    label_id: ItemId,
    bar_id: BarId,

    track_volume_id: ItemId,

    normal_tb_elements: Vec<ItemId>,
    track_tb_elements: Vec<ItemId>,

    last_selected_track: Option<Guid>,
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
        // NOTE this gets run every time an action is run

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

            let project = Reaper::get().current_project();

            let selected_count =
                project.selected_track_count(MasterTrackBehavior::ExcludeMasterTrack);
            if selected_count == 0 {
                state.change_mode(Mode::Normal);
                update_label("No tracks selected")
            } else {
                state.change_mode(Mode::Track);
                update_label(&format!("{} tracks selected", selected_count))
            }

            let track = Track::new(args.track, None);
            let id = track.guid();
            if args.is_selected {
                state.tb.update_slider(
                    &state.track_volume_id,
                    track.volume().soft_normalized_value(),
                );
                state.last_selected_track = Some(*id);
            }
        }
    }

    fn set_surface_volume(&self, args: reaper_medium::SetSurfaceVolumeArgs) {
        unsafe {
            let Some(state) = &mut STATE else {
                return;
            };

            let reaper = Reaper::get().medium_reaper();

            if reaper
                .get_media_track_info_value(args.track, reaper_medium::TrackAttributeKey::Selected)
                as u64
                == 0
            {
                return;
            }

            let track = Track::new(args.track, None);
            state.last_selected_track = Some(*track.guid());

            state.tb.update_slider(
                &state.track_volume_id,
                track.volume().soft_normalized_value(),
            );
        }
    }
}

fn mute_selected(_: &ItemId) {
    let reaper = Reaper::get();
    let project = reaper.current_project();

    let Some(track) =
            project.first_selected_track(reaper_medium::MasterTrackBehavior::ExcludeMasterTrack)
        else {
            return;
        };

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
fn solo_selected(_: &ItemId) {
    let reaper = Reaper::get();
    let project = reaper.current_project();

    let Some(track) =
            project.first_selected_track(reaper_medium::MasterTrackBehavior::ExcludeMasterTrack)
        else {
            return;
        };

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
fn volume_selected(_: &ItemId, v: f64) {
    unsafe {
        let Some(state) = &mut STATE else {
            return;
        };

        let reaper = Reaper::get();
        let project = reaper.current_project();

        let Some(track) =
            state.last_selected_track.and_then(|t| project.track_by_guid(&t).ok())
        else {
            return;
        };

        let vol = Volume::try_from_soft_normalized_value(v.min(4.0)).unwrap_or(Volume::MIN);
        track.set_volume(
            vol,
            reaper_medium::GangBehavior::AllowGang,
            reaper_high::GroupingBehavior::UseGrouping,
        );
    }
}
