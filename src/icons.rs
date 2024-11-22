use std::sync::{LazyLock, OnceLock};

use cushy::{
    fonts::{FontCollection, LoadedFont},
    styles::{
        components::{FontFamily, FontWeight, LineHeight},
        DynamicComponent, FamilyOwned, FontFamilyList, Weight,
    },
    widget::MakeWidget,
    widgets::{button::ButtonKind, Button},
    Cushy,
};

static FONT: OnceLock<LoadedFont> = OnceLock::new();

pub fn load_fonts(col: &FontCollection) {
    let font = col.push_unloadable(
        include_bytes!("../fonts/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf").into(),
    );
    FONT.set(font).map_err(|_| ()).unwrap();
}

fn font_component() -> DynamicComponent {
    DynamicComponent::new({
        move |context| {
            let font = &FONT.get().unwrap();
            let faces = context.loaded_font_faces(font);
            let face = faces.first()?;
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

pub fn iconbtn(str: &str) -> Button {
    icon(str).into_button().kind(ButtonKind::Transparent)
}

pub trait IntoIcon {
    fn into_icon(self) -> impl MakeWidget;
    fn into_iconbtn(self) -> Button
    where
        Self: Sized,
    {
        self.into_icon().into_button().kind(ButtonKind::Transparent)
    }
}

impl IntoIcon for &str {
    fn into_icon(self) -> impl MakeWidget {
        icon(self)
    }
}

pub const PLAY: &str = "\u{e037}";
pub const PAUSE: &str = "\u{e034}";
pub const REPEAT: &str = "\u{e040}";
pub const REPEAT_ON: &str = "\u{e9d6}";
pub const REPEAT_ONE: &str = "\u{e041}";
pub const REPEAT_ONE_ON: &str = "\u{e9d7}";
pub const SHUFFLE: &str = "\u{e043}";
pub const SHUFFLE_ON: &str = "\u{e9e1}";
pub const SKIP_NEXT: &str = "\u{e044}";
pub const SKIP_PREVIOUS: &str = "\u{e045}";
pub const QUEUE_MUSIC: &str = "\u{e03d}";
pub const EXPLICIT: &str = "\u{e01e}";
pub const ALBUM: &str = "\u{e019}";
pub const LYRICS: &str = "\u{ec0b}";
pub const MUSIC_CAST: &str = "\u{eb1a}";
