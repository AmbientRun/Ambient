use ambient_api::{
    element::{use_state, Setter},
    prelude::*,
    ui::use_keyboard_input,
};

pub fn use_hotkey_toggle(
    hooks: &mut Hooks,
    target_keycode: VirtualKeyCode,
) -> (bool, Setter<bool>) {
    let (toggle, set_toggle) = use_state(hooks, false);
    use_keyboard_input(hooks, {
        let set_toggle = set_toggle.clone();
        move |_, keycode, modifiers, pressed| {
            if modifiers == ModifiersState::empty() && keycode == Some(target_keycode) && !pressed {
                set_toggle(!toggle);
            }
        }
    });

    (toggle, set_toggle)
}
