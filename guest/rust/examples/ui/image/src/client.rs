use ambient_api::{
    core::{
        layout::components::space_between_items,
        rect::components::{
            background_color, border_color, border_radius, border_thickness,
            size_from_background_image,
        },
    },
    prelude::*,
    ui::ImageFromUrl,
};
use packages::this::assets;

#[element_component]
fn App(_hooks: &mut Hooks) -> Element {
    Group::el([FlowColumn::el([
        ImageFromUrl {
            url: assets::url("squirrel.png"),
        }
        .el()
        .with(size_from_background_image(), ()),
        FlowRow::el([
            ImageFromUrl {
                url: "https://commons.wikimedia.org/w/index.php?title=Special:Redirect/file/Bucephala-albeola-010.jpg"
                    .to_string(),
            }
            .el(),
            ImageFromUrl {
                url: "https://commons.wikimedia.org/w/index.php?title=Special:Redirect/file/Alpha_transparency_image.png"
                    .to_string(),
            }
            .el()
            .with(background_color(), vec4(1., 0., 1., 1.)),
            ImageFromUrl {
                url: "https://commons.wikimedia.org/w/index.php?title=Special:Redirect/file/Bucephala-albeola-010.jpg"
                    .to_string(),
            }
            .el()
            .with(border_radius(), Vec4::ONE * 10.)
            .with(border_color(), vec4(0., 1., 0., 1.))
            .with(border_thickness(), 10.),
            ImageFromUrl {
                url: "invalid url".to_string(),
            }
            .el(),
        ]).with(space_between_items(), 10.)
    ])
    .with(space_between_items(), 10.)
    .with_padding_even(STREET)])
}

#[main]
pub fn main() {
    App.el().spawn_interactive();
}
