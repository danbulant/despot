use cushy::widget::{Widget, WidgetRef};


#[derive(Debug)]
pub struct OwnedWidget<W>(OwnedWidgetState<W>);

#[derive(Debug)]
enum OwnedWidgetState<W> {
    Unmade(W),
    Making,
    Made(WidgetRef),
}

impl<W> OwnedWidget<W>
where
    W: Widget,
{
    pub const fn new(widget: W) -> Self {
        Self(OwnedWidgetState::Unmade(widget))
    }

    pub fn make_if_needed(&mut self) -> &mut WidgetRef {
        if matches!(&self.0, OwnedWidgetState::Unmade(_)) {
            let OwnedWidgetState::Unmade(widget) =
                std::mem::replace(&mut self.0, OwnedWidgetState::Making)
            else {
                unreachable!("just matched")
            };

            self.0 = OwnedWidgetState::Made(WidgetRef::new(widget));
        }

        self.expect_made_mut()
    }

    pub fn expect_made(&self) -> &WidgetRef {
        let OwnedWidgetState::Made(widget) = &self.0 else {
            unreachable!("widget made")
        };
        widget
    }

    pub fn expect_made_mut(&mut self) -> &mut WidgetRef {
        let OwnedWidgetState::Made(widget) = &mut self.0 else {
            unreachable!("widget made")
        };
        widget
    }

    pub fn expect_unmade_mut(&mut self) -> &mut W {
        let OwnedWidgetState::Unmade(widget) = &mut self.0 else {
            unreachable!("widget unmade")
        };
        widget
    }
}