use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, SubmitEvent};
use yew::{function_component, html, Callback, Children, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct TextInputProps {
    pub name: String,
    pub onchanged: Callback<String>,
}

#[function_component]
pub fn TextInput(props: &TextInputProps) -> Html {
    let on_changed = props.onchanged.clone();

    let on_changed = Callback::from(move |event: Event| {
        event.prevent_default();
        let value = event
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            .map(|v| v.value());

        if let Some(value) = value {
            on_changed.emit(value)
        }
    });

    html! {
        <input type="text" name={props.name.clone()} onchange={on_changed} placeholder={props.name.clone()}/>
    }
}

#[derive(Properties, PartialEq)]
pub struct FormProps {
    pub onsubmit: Callback<()>,
    pub children: Children,
}

#[function_component]
pub fn Form(props: &FormProps) -> Html {
    let onsubmit = props.onsubmit.clone();
    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();
        if event.target().is_some() {
            onsubmit.emit(());
        }
    });

    html! {
        <form method="post" onsubmit={onsubmit}>
            { for props.children.iter() }
        <input type="submit" value = "Submit"/>
        </form>
    }
}
