use freya::prelude::*;

#[derive(PartialEq)]
pub struct LapBar;

impl Component for LapBar {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::window_percent(33.0))
            .position(Position::new_global().right(0.0).top(0.0))
            .child(label().text("Lap"))
            .padding(4.0)
            .border(Some(
                Border::new()
                    .width(1.0)
                    .fill(Color::BLACK)
                    .alignment(BorderAlignment::Inner),
            ))
    }
}
