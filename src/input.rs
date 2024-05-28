use bevy::{
    prelude::*,
};

use crate::settings::{
    UserSettings,
    KeyBindings
};
use crate::lane::{
    Lane,
    LaneMap,
};

/// Represents a user attempting to complete the note in a lane.
#[derive(Event)]
#[derive(Debug,Clone)]
pub struct LaneHit {
    /// Lane that was hit
    lane: Lane,
    /// When the key was pressed
    time_of_hit: f32,
}
impl LaneHit {
    pub fn lane(&self) -> Lane {
        self.lane
    }
    pub fn time_of_hit(&self) -> f32 {
        self.time_of_hit
    }
}

#[derive(Resource)]
#[allow(non_snake_case)]
pub struct InputManager {
    /// Keybindings
    lane_hit_keycodes: LaneMap<KeyCode>,
}

#[allow(non_snake_case)]
fn setup_input_manager(mut commands: Commands, settings: Res<UserSettings>) {
    let KeyBindings {
        lane_hit_L1,
        lane_hit_L2,
        lane_hit_R1,
        lane_hit_R2,
        ..
    } = &settings.keybindings;

    
    let lane_hit_keycodes = LaneMap::from([
        to_keycode(lane_hit_L1),
        to_keycode(lane_hit_L2),
        to_keycode(lane_hit_R1),
        to_keycode(lane_hit_R2),
    ]);

    commands.insert_resource(InputManager {
        lane_hit_keycodes
    });
}

fn listen_for_input(
    time: Res<Time>,
    input_mgr: Res<InputManager>,
    keys: Res<ButtonInput<KeyCode>>,
    mut lane_hit_events: EventWriter<LaneHit>,
) {
    let now = time.elapsed().as_secs_f32();

    input_mgr
        .lane_hit_keycodes
        .iter()
        .filter(|(_lane, &keycode)| keys.just_pressed(keycode))
        .map(|(lane, _keycode)| LaneHit {
            lane,
            time_of_hit: now
        })
        .for_each(|ev| {
            log::debug!("Sending lane hit event");
            lane_hit_events.send(ev);
        });

}

fn to_keycode(name: &str) -> KeyCode {
    let key = match name.to_lowercase().as_str() {
        "backquote" => KeyCode::Backquote,
        "backslash" => KeyCode::Backslash,
        "bracketleft" => KeyCode::BracketLeft,
        "bracketright" => KeyCode::BracketRight,
        "comma" => KeyCode::Comma,
        "0" => KeyCode::Digit0,
        "1" => KeyCode::Digit1,
        "2" => KeyCode::Digit2,
        "3" => KeyCode::Digit3,
        "4" => KeyCode::Digit4,
        "5" => KeyCode::Digit5,
        "6" => KeyCode::Digit6,
        "7" => KeyCode::Digit7,
        "8" => KeyCode::Digit8,
        "9" => KeyCode::Digit9,
        "equal" => KeyCode::Equal,
        "intlbackslash" => KeyCode::IntlBackslash,
        "intlro" => KeyCode::IntlRo,
        "intlyen" => KeyCode::IntlYen,
        "a" => KeyCode::KeyA,
        "b" => KeyCode::KeyB,
        "c" => KeyCode::KeyC,
        "d" => KeyCode::KeyD,
        "e" => KeyCode::KeyE,
        "f" => KeyCode::KeyF,
        "g" => KeyCode::KeyG,
        "h" => KeyCode::KeyH,
        "i" => KeyCode::KeyI,
        "j" => KeyCode::KeyJ,
        "k" => KeyCode::KeyK,
        "l" => KeyCode::KeyL,
        "m" => KeyCode::KeyM,
        "n" => KeyCode::KeyN,
        "o" => KeyCode::KeyO,
        "p" => KeyCode::KeyP,
        "q" => KeyCode::KeyQ,
        "r" => KeyCode::KeyR,
        "s" => KeyCode::KeyS,
        "t" => KeyCode::KeyT,
        "u" => KeyCode::KeyU,
        "v" => KeyCode::KeyV,
        "w" => KeyCode::KeyW,
        "x" => KeyCode::KeyX,
        "y" => KeyCode::KeyY,
        "z" => KeyCode::KeyZ,
        "minus" => KeyCode::Minus,
        "period" => KeyCode::Period,
        "quote" => KeyCode::Quote,
        "semicolon" => KeyCode::Semicolon,
        "slash" => KeyCode::Slash,
        "altleft" => KeyCode::AltLeft,
        "altright" => KeyCode::AltRight,
        "backspace" => KeyCode::Backspace,
        "capslock" => KeyCode::CapsLock,
        "contextmenu" => KeyCode::ContextMenu,
        "controlleft" => KeyCode::ControlLeft,
        "controlright" => KeyCode::ControlRight,
        "enter" => KeyCode::Enter,
        "superleft" => KeyCode::SuperLeft,
        "superright" => KeyCode::SuperRight,
        "shiftleft" => KeyCode::ShiftLeft,
        "shiftright" => KeyCode::ShiftRight,
        "space" => KeyCode::Space,
        "tab" => KeyCode::Tab,
        "convert" => KeyCode::Convert,
        "kanamode" => KeyCode::KanaMode,
        "lang1" => KeyCode::Lang1,
        "lang2" => KeyCode::Lang2,
        "lang3" => KeyCode::Lang3,
        "lang4" => KeyCode::Lang4,
        "lang5" => KeyCode::Lang5,
        "nonconvert" => KeyCode::NonConvert,
        "delete" => KeyCode::Delete,
        "end" => KeyCode::End,
        "help" => KeyCode::Help,
        "home" => KeyCode::Home,
        "insert" => KeyCode::Insert,
        "pagedown" => KeyCode::PageDown,
        "pageup" => KeyCode::PageUp,
        "arrowdown" => KeyCode::ArrowDown,
        "arrowleft" => KeyCode::ArrowLeft,
        "arrowright" => KeyCode::ArrowRight,
        "arrowup" => KeyCode::ArrowUp,
        "numlock" => KeyCode::NumLock,
        "numpad0" => KeyCode::Numpad0,
        "numpad1" => KeyCode::Numpad1,
        "numpad2" => KeyCode::Numpad2,
        "numpad3" => KeyCode::Numpad3,
        "numpad4" => KeyCode::Numpad4,
        "numpad5" => KeyCode::Numpad5,
        "numpad6" => KeyCode::Numpad6,
        "numpad7" => KeyCode::Numpad7,
        "numpad8" => KeyCode::Numpad8,
        "numpad9" => KeyCode::Numpad9,
        "numpadadd" => KeyCode::NumpadAdd,
        "numpadbackspace" => KeyCode::NumpadBackspace,
        "numpadclear" => KeyCode::NumpadClear,
        "numpadclearentry" => KeyCode::NumpadClearEntry,
        "numpadcomma" => KeyCode::NumpadComma,
        "numpaddecimal" => KeyCode::NumpadDecimal,
        "numpaddivide" => KeyCode::NumpadDivide,
        "numpadenter" => KeyCode::NumpadEnter,
        "numpadequal" => KeyCode::NumpadEqual,
        "numpadhash" => KeyCode::NumpadHash,
        "numpadmemoryadd" => KeyCode::NumpadMemoryAdd,
        "numpadmemoryclear" => KeyCode::NumpadMemoryClear,
        "numpadmemoryrecall" => KeyCode::NumpadMemoryRecall,
        "numpadmemorystore" => KeyCode::NumpadMemoryStore,
        "numpadmemorysubtract" => KeyCode::NumpadMemorySubtract,
        "numpadmultiply" => KeyCode::NumpadMultiply,
        "numpadparenleft" => KeyCode::NumpadParenLeft,
        "numpadparenright" => KeyCode::NumpadParenRight,
        "numpadstar" => KeyCode::NumpadStar,
        "numpadsubtract" => KeyCode::NumpadSubtract,
        "escape" => KeyCode::Escape,
        "fn" => KeyCode::Fn,
        "fnlock" => KeyCode::FnLock,
        "printscreen" => KeyCode::PrintScreen,
        "scrolllock" => KeyCode::ScrollLock,
        "pause" => KeyCode::Pause,
        "browserback" => KeyCode::BrowserBack,
        "browserfavorites" => KeyCode::BrowserFavorites,
        "browserforward" => KeyCode::BrowserForward,
        "browserhome" => KeyCode::BrowserHome,
        "browserrefresh" => KeyCode::BrowserRefresh,
        "browsersearch" => KeyCode::BrowserSearch,
        "browserstop" => KeyCode::BrowserStop,
        "eject" => KeyCode::Eject,
        "launchapp1" => KeyCode::LaunchApp1,
        "launchapp2" => KeyCode::LaunchApp2,
        "launchmail" => KeyCode::LaunchMail,
        "mediaplaypause" => KeyCode::MediaPlayPause,
        "mediaselect" => KeyCode::MediaSelect,
        "mediastop" => KeyCode::MediaStop,
        "mediatracknext" => KeyCode::MediaTrackNext,
        "mediatrackprevious" => KeyCode::MediaTrackPrevious,
        "power" => KeyCode::Power,
        "sleep" => KeyCode::Sleep,
        "audiovolumedown" => KeyCode::AudioVolumeDown,
        "audiovolumemute" => KeyCode::AudioVolumeMute,
        "audiovolumeup" => KeyCode::AudioVolumeUp,
        "wakeup" => KeyCode::WakeUp,
        "meta" => KeyCode::Meta,
        "hyper" => KeyCode::Hyper,
        "turbo" => KeyCode::Turbo,
        "abort" => KeyCode::Abort,
        "resume" => KeyCode::Resume,
        "suspend" => KeyCode::Suspend,
        "again" => KeyCode::Again,
        "copy" => KeyCode::Copy,
        "cut" => KeyCode::Cut,
        "find" => KeyCode::Find,
        "open" => KeyCode::Open,
        "paste" => KeyCode::Paste,
        "props" => KeyCode::Props,
        "select" => KeyCode::Select,
        "undo" => KeyCode::Undo,
        "hiragana" => KeyCode::Hiragana,
        "katakana" => KeyCode::Katakana,
        "f1" => KeyCode::F1,
        "f2" => KeyCode::F2,
        "f3" => KeyCode::F3,
        "f4" => KeyCode::F4,
        "f5" => KeyCode::F5,
        "f6" => KeyCode::F6,
        "f7" => KeyCode::F7,
        "f8" => KeyCode::F8,
        "f9" => KeyCode::F9,
        "f10" => KeyCode::F10,
        "f11" => KeyCode::F11,
        "f12" => KeyCode::F12,
        "f13" => KeyCode::F13,
        "f14" => KeyCode::F14,
        "f15" => KeyCode::F15,
        "f16" => KeyCode::F16,
        "f17" => KeyCode::F17,
        "f18" => KeyCode::F18,
        "f19" => KeyCode::F19,
        "f20" => KeyCode::F20,
        "f21" => KeyCode::F21,
        "f22" => KeyCode::F22,
        "f23" => KeyCode::F23,
        "f24" => KeyCode::F24,
        "f25" => KeyCode::F25,
        "f26" => KeyCode::F26,
        "f27" => KeyCode::F27,
        "f28" => KeyCode::F28,
        "f29" => KeyCode::F29,
        "f30" => KeyCode::F30,
        "f31" => KeyCode::F31,
        "f32" => KeyCode::F32,
        "f33" => KeyCode::F33,
        "f34" => KeyCode::F34,
        "f35" => KeyCode::F35,
        s => {
            panic!("Text `{}' does not specify a valid keycode", s);
        }
    };
    key
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Building Input plugin");
        app
            .add_event::<LaneHit>()
            .add_systems(Startup, setup_input_manager)
            .add_systems(PreUpdate, listen_for_input) // important that input happens the frame it's detected
        ;
    }
}
