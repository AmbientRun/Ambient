use ambient_api::{element::Setter, prelude::*};

pub fn use_hotkey_toggle(
    hooks: &mut Hooks,
    target_keycode: VirtualKeyCode,
) -> (bool, Setter<bool>) {
    let (toggle, set_toggle) = hooks.use_state(false);
    hooks.use_keyboard_input({
        let set_toggle = set_toggle.clone();
        move |_, keycode, modifiers, pressed| {
            if modifiers == ModifiersState::empty() && keycode == Some(target_keycode) && !pressed {
                set_toggle(!toggle);
            }
        }
    });

    (toggle, set_toggle)
}
