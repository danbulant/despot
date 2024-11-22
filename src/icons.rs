use std::sync::{LazyLock, OnceLock};

use cushy::{
    fonts::{FontCollection, LoadedFont},
    styles::{
        components::{FontFamily, FontWeight, LineHeight},
        DynamicComponent, FamilyOwned, FontFamilyList, Weight,
    },
    widget::MakeWidget,
    Cushy,
};

static FONT: OnceLock<LoadedFont> = OnceLock::new();

pub fn load_fonts(/*cushy: &Cushy*/) -> FontCollection {
    let fonts = FontCollection::default();
    let font_data = include_bytes!("../fonts/MaterialIcons-Regular.ttf").to_vec();
    dbg!(font_data.len());
    let font = fonts.push_unloadable(
        // include_bytes!("../fonts/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf").into(),
        font_data,
    );
    // /home/dan/projects/despot/fonts/MaterialIcons-Regular.ttf
    println!("loaded");
    FONT.set(font).map_err(|_| ()).unwrap();
    fonts
}

fn font_component() -> DynamicComponent {
    DynamicComponent::new({
        move |context| {
            let font = &FONT.get().unwrap();
            // dbg!(font);
            let faces = context.loaded_font_faces(font);
            dbg!(&faces);
            let face = faces.first()?;
            dbg!(&face);
            Some(cushy::styles::Component::custom(FontFamilyList::from(
                vec![FamilyOwned::Name(face.families[0].0.clone())],
            )))
        }
    })
}

fn font_weight_component() -> DynamicComponent {
    DynamicComponent::new({
        move |context| {
            let font = &FONT.get().unwrap();
            let faces = context.loaded_font_faces(font);
            let face = faces.first()?;
            Some(cushy::styles::Component::FontWeight(face.weight))
        }
    })
}

static FONT_FAMILY_COMPONENT: LazyLock<DynamicComponent> = LazyLock::new(font_component);
static FONT_WEIGHT_COMPONENT: LazyLock<DynamicComponent> = LazyLock::new(font_weight_component);

pub fn icon(str: &str) -> impl MakeWidget {
    str.with_dynamic(&FontFamily, FONT_FAMILY_COMPONENT.clone())
        .with_dynamic(&FontWeight, FONT_WEIGHT_COMPONENT.clone())
}

pub const SHUFFLE: &str = "\u{e043}";
pub const SKIP_PREVIOUS: &str = "\u{e045}";
