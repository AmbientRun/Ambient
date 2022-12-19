use elements_core::asset_cache;
use elements_ecs::World;
use elements_element::{Element, ElementComponent, ElementComponentExt, Hooks};
use elements_std::{
    asset_url::{select_asset, AssetUrl, AssetUrlCollection, GetAssetType}, Cb
};

use crate::{
    align_vertical, space_between_items, Alert, Align, Button, ButtonStyle, Editor, EditorOpts, FlowRow, ScreenContainer, Text, STREET
};

impl<T: GetAssetType + 'static> Editor for AssetUrl<T> {
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _opts: EditorOpts) -> Element {
        AssetUrlEditor { value, on_change }.el()
    }
}
#[derive(Debug, Clone)]
pub struct AssetUrlEditor<T: GetAssetType> {
    pub value: AssetUrl<T>,
    pub on_change: Option<Cb<dyn Fn(AssetUrl<T>) + Sync + Send>>,
}
impl<T: GetAssetType + 'static> ElementComponent for AssetUrlEditor<T> {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        if let Some(on_change) = on_change {
            FlowRow::el([
                Text::el(value.display_name.as_ref().unwrap_or(&value.url)),
                Button::new("\u{f74e} Browse", move |world| {
                    let on_change = on_change.clone();
                    select_asset(world.resource(asset_cache()), T::asset_type(), move |asset_url| {
                        if let Some(url) = asset_url.random() {
                            on_change(AssetUrl {
                                url: url.to_string(),
                                display_name: asset_url.name().map(|x| x.to_string()),
                                asset_type: std::marker::PhantomData,
                            });
                        }
                    });
                })
                .style(ButtonStyle::Flat)
                .el(),
            ])
            .set(align_vertical(), Align::Center)
            .set(space_between_items(), STREET)
        } else {
            Text::el(value.url)
        }
    }
}

impl<T: GetAssetType + 'static> Editor for AssetUrlCollection<T> {
    fn editor(value: Self, on_change: Option<Cb<dyn Fn(Self) + Sync + Send>>, _opts: EditorOpts) -> Element {
        AssetUrlCollectionEditor { value, on_change }.el()
    }
}
#[derive(Debug, Clone)]
pub struct AssetUrlCollectionEditor<T: GetAssetType> {
    pub value: AssetUrlCollection<T>,
    pub on_change: Option<Cb<dyn Fn(AssetUrlCollection<T>) + Sync + Send>>,
}
impl<T: GetAssetType + 'static> ElementComponent for AssetUrlCollectionEditor<T> {
    fn render(self: Box<Self>, _world: &mut World, hooks: &mut Hooks) -> Element {
        let Self { value, on_change } = *self;
        if let Some(on_change) = on_change {
            FlowRow::el([
                Text::el(value.display_name.as_ref().cloned().unwrap_or_else(|| format!("{:?}", value.urls))),
                Button::new("\u{f74e} Browse", move |world| {
                    let on_change = on_change.clone();
                    select_asset(world.resource(asset_cache()), T::asset_type(), move |asset_url| {
                        on_change(AssetUrlCollection {
                            urls: asset_url.all().into_iter().cloned().collect(),
                            display_name: asset_url.name().map(|x| x.to_string()),
                            asset_type: std::marker::PhantomData,
                        });
                    });
                })
                .style(ButtonStyle::Flat)
                .el(),
            ])
            .set(align_vertical(), Align::Center)
            .set(space_between_items(), STREET)
        } else {
            Text::el(format!("{:?}", value.urls))
        }
    }
}
