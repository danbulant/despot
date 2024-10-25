use cushy::{figures::{Round, ScreenScale, Size, Zero}, styles::{Dimension, DimensionRange, Edges}, value::{Destination, Dynamic, ForEach, IntoDynamic, MapEach, Source}, widget::{MakeWidget, WidgetList}, widgets::{Container, Scroll, Stack}};
use crate::widgets::probe::ScalingProbe;

pub trait VirtualList {
    fn item_height(&self) -> impl IntoDynamic<Dimension>;
    // fn width(&self) -> impl IntoDynamic<DimensionRange>;
    fn item_count(&self) -> impl IntoDynamic<usize>;
    fn widget_at(&self, index: usize) -> impl MakeWidget;
}

pub fn virtual_list<T>(list: T) -> impl MakeWidget
where
    T: VirtualList + Send + 'static
{
    let contents = Dynamic::default();
    let stack = Stack::rows(contents.clone());
    let padding = Dynamic::default();
    let container = Container::new(stack).transparent().pad_by(padding.clone());
    let scroll = Scroll::vertical(container);

    // Current scroll position
    let current_scroll = scroll.scroll.clone().map_each(|scroll| scroll.y);
    // height of the scroll widget
    let visible_size = scroll.control_size().map_each(|size| size.height);
    // max scroll position. Height of contents is max_scroll + visible_size
    // let max_scroll = scroll.max_scroll().map_each(|size| size.y);

    let item_height = list.item_height().into_dynamic();
    let item_count = list.item_count().into_dynamic();

    let probe = ScalingProbe::new(scroll);
    let scale = probe.scale();

    // let width = list.width().into_dynamic();

    // (&width, &item_height, &item_count, &scale).map_each(|(width, item_height, item_count, scale)| {
    //     Size::new(*width, *item_height.into_upx(*scale) * *item_count as f32)
    // });

    let handle = (&current_scroll, &item_height, &item_count, &scale, &visible_size).for_each(move |(current_scroll, item_height, item_count, scale, visible_size)| {
        let start = (*current_scroll / item_height.into_upx(*scale)).floor().get();
        let end = ((*current_scroll + *visible_size) / item_height.into_upx(*scale)).ceil().get().min(*item_count as _);
        println!("Start: {}, End: {}", start, end);

        let list = (start as usize..end as usize).map(|index| list.widget_at(index));

        let padding_start = *item_height * start as i32;
        let items_end = (*item_count as u32).saturating_sub(end);
        let padding_end = *item_height * items_end as i32;

        padding.set(Edges::ZERO.with_top(padding_start).with_bottom(padding_end));

        contents.set(WidgetList::from_iter(list));
    });
    handle.persist();

    probe
}