use std::sync::{LazyLock, OnceLock};

use cushy::{
    fonts::LoadedFont,
    styles::{
        components::{FontFamily, FontWeight, LineHeight},
        DynamicComponent, FamilyOwned, FontFamilyList, Weight,
    },
    widget::MakeWidget,
    Cushy,
};

static FONT: OnceLock<LoadedFont> = OnceLock::new();

pub fn load_fonts(cushy: &Cushy) {
    let font = cushy.fonts().push_unloadable(
        // include_bytes!("../fonts/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf").into(),
        include_bytes!("../fonts/MaterialIcons-Regular.ttf").into(),
    );
    println!("loaded");
    FONT.set(font).map_err(|_| ()).unwrap();
}

fn font_component() -> DynamicComponent {
    DynamicComponent::new({
        move |context| {
            let font = &FONT.get().unwrap();
            // dbg!(font);
            let face = context.loaded_font_faces(font).first()?;
            dbg!(&face);
            Some(cushy::styles::Component::custom(FontFamilyList::from(
                vec![FamilyOwned::Name(face.families[0].0.clone())],
            )))
        }
    })
}

static FONT_FAMILY_COMPONENT: LazyLock<DynamicComponent> = LazyLock::new(font_component);

pub fn icon(str: &str) -> impl MakeWidget {
    str.with_dynamic(&FontFamily, FONT_FAMILY_COMPONENT.clone())
        .with(&FontWeight, Weight::NORMAL)
}

pub const SHUFFLE: &str = "\u{e043}";
pub const SKIP_PREVIOUS: &str = "\u{e045}";
