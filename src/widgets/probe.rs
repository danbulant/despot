use cushy::{figures::Fraction, value::{Destination, Dynamic, DynamicReader, IntoReadOnly, ReadOnly}, widget::{MakeWidget, Widget, WidgetRef}};

#[derive(Debug)]
pub struct ScalingProbe {
    child: WidgetRef,
    scale: Dynamic<Fraction>
}

impl ScalingProbe {
    pub fn new(child: impl MakeWidget) -> Self {
        Self {
            child: WidgetRef::new(child),
            scale: Dynamic::new(Fraction::new_whole(1))
        }
    }

    pub fn scale(&self) -> DynamicReader<Fraction> {
        self.scale.create_reader()
    }
}

impl Widget for ScalingProbe {
    fn redraw(&mut self, context: &mut cushy::context::GraphicsContext<'_, '_, '_, '_>) {
        self.scale.set(context.gfx.scale());
        context.for_other(&self.child).expect("A child").redraw();
    }
    fn layout(
        &mut self,
        available_space: cushy::figures::Size<cushy::ConstraintLimit>,
        context: &mut cushy::context::LayoutContext<'_, '_, '_, '_>,
    ) -> cushy::figures::Size<cushy::figures::units::UPx> {
        let child = self.child.mounted(context);
        context.for_other(&child).layout(available_space)
    }
}