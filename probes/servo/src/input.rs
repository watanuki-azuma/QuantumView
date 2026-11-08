/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// The fixture needs text editing, navigation and Japanese input keys.
// This is the corresponding subset of Servo v0.7.0's desktop/keyutils.rs.
use servo::{Code, Cursor, Key, KeyState, KeyboardEvent, Location, Modifiers, NamedKey};
use winit::event::{ElementState, KeyEvent, MouseButton as WinitMouseButton};
use winit::keyboard::{Key as WinitKey, KeyLocation, ModifiersState, NamedKey as WinitNamedKey};
use winit::window::{CursorIcon, Window};

pub fn keyboard_event(
    event: &KeyEvent,
    modifiers: ModifiersState,
    composing: bool,
) -> KeyboardEvent {
    let key = match &event.logical_key {
        WinitKey::Character(text) => Key::Character(text.to_string()),
        WinitKey::Named(WinitNamedKey::Space) => Key::Character(" ".into()),
        WinitKey::Named(named) => Key::Named(match named {
            WinitNamedKey::Enter => NamedKey::Enter,
            WinitNamedKey::Tab => NamedKey::Tab,
            WinitNamedKey::Backspace => NamedKey::Backspace,
            WinitNamedKey::Delete => NamedKey::Delete,
            WinitNamedKey::Insert => NamedKey::Insert,
            WinitNamedKey::Escape => NamedKey::Escape,
            WinitNamedKey::ArrowUp => NamedKey::ArrowUp,
            WinitNamedKey::ArrowDown => NamedKey::ArrowDown,
            WinitNamedKey::ArrowLeft => NamedKey::ArrowLeft,
            WinitNamedKey::ArrowRight => NamedKey::ArrowRight,
            WinitNamedKey::Home => NamedKey::Home,
            WinitNamedKey::End => NamedKey::End,
            WinitNamedKey::PageUp => NamedKey::PageUp,
            WinitNamedKey::PageDown => NamedKey::PageDown,
            WinitNamedKey::Shift => NamedKey::Shift,
            WinitNamedKey::Control => NamedKey::Control,
            WinitNamedKey::Alt => NamedKey::Alt,
            WinitNamedKey::AltGraph => NamedKey::AltGraph,
            WinitNamedKey::Super | WinitNamedKey::Meta => NamedKey::Meta,
            WinitNamedKey::CapsLock => NamedKey::CapsLock,
            WinitNamedKey::Convert => NamedKey::Convert,
            WinitNamedKey::NonConvert => NamedKey::NonConvert,
            WinitNamedKey::Hiragana => NamedKey::Hiragana,
            WinitNamedKey::Katakana => NamedKey::Katakana,
            WinitNamedKey::HiraganaKatakana => NamedKey::HiraganaKatakana,
            WinitNamedKey::ZenkakuHankaku => NamedKey::ZenkakuHankaku,
            WinitNamedKey::KanaMode => NamedKey::KanaMode,
            WinitNamedKey::KanjiMode => NamedKey::KanjiMode,
            WinitNamedKey::Romaji => NamedKey::Romaji,
            WinitNamedKey::Process => NamedKey::Process,
            _ => NamedKey::Unidentified,
        }),
        _ => Key::Named(NamedKey::Unidentified),
    };
    let mut flags = Modifiers::empty();
    flags.set(Modifiers::SHIFT, modifiers.shift_key());
    flags.set(Modifiers::CONTROL, modifiers.control_key());
    flags.set(Modifiers::ALT, modifiers.alt_key());
    flags.set(Modifiers::META, modifiers.super_key());
    KeyboardEvent::new_without_event(
        match event.state {
            ElementState::Pressed => KeyState::Down,
            ElementState::Released => KeyState::Up,
        },
        key,
        // Physical scan-code shortcuts are outside this text-editing probe.
        Code::Unidentified,
        match event.location {
            KeyLocation::Standard => Location::Standard,
            KeyLocation::Left => Location::Left,
            KeyLocation::Right => Location::Right,
            KeyLocation::Numpad => Location::Numpad,
        },
        flags,
        event.repeat,
        composing,
    )
}

pub fn mouse_button(button: WinitMouseButton) -> servo::MouseButton {
    match button {
        WinitMouseButton::Left => servo::MouseButton::Primary,
        WinitMouseButton::Middle => servo::MouseButton::Auxiliary,
        WinitMouseButton::Right => servo::MouseButton::Secondary,
        WinitMouseButton::Back => servo::MouseButton::Back,
        WinitMouseButton::Forward => servo::MouseButton::Forward,
        WinitMouseButton::Other(button) => servo::MouseButton::Other(button),
    }
}

pub fn set_cursor(window: &Window, cursor: Cursor) {
    let icon = match cursor {
        Cursor::None => {
            window.set_cursor_visible(false);
            return;
        }
        Cursor::Pointer => CursorIcon::Pointer,
        Cursor::Text => CursorIcon::Text,
        Cursor::VerticalText => CursorIcon::VerticalText,
        Cursor::Crosshair => CursorIcon::Crosshair,
        Cursor::Move => CursorIcon::Move,
        Cursor::NotAllowed => CursorIcon::NotAllowed,
        Cursor::Grab => CursorIcon::Grab,
        Cursor::Grabbing => CursorIcon::Grabbing,
        Cursor::EwResize | Cursor::ColResize => CursorIcon::EwResize,
        Cursor::NsResize | Cursor::RowResize => CursorIcon::NsResize,
        _ => CursorIcon::Default,
    };
    window.set_cursor(icon);
    window.set_cursor_visible(true);
}
