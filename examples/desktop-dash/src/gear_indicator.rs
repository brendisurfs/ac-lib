use freya::{prelude::*, radio::use_radio};

use crate::DataChannel;

#[derive(PartialEq)]
pub struct GearIndicator;

impl Component for GearIndicator {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(DataChannel::CarInfo);

        rect()
            .border(Some(
                Border::new()
                    .width(1.0)
                    .fill(Color::BLACK)
                    .alignment(BorderAlignment::Inner),
            ))
            .child(
                label().font_size(128.0).text(
                    radio
                        .read()
                        .car_info
                        .as_ref()
                        .map(|v| v.gear)
                        .unwrap_or(0)
                        .to_string(),
                ),
            )
    }
}
