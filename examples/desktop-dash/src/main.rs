mod gear_indicator;
mod lap_bar;
use std::sync::Arc;

use ac_lib::parser::ACEvent;
use ac_lib::parser::RTCarInfo;
use ac_lib::parser::RTLapInfo;
use freya::{prelude::*, radio::*};

use ac_lib::Client;
use ac_lib::parser::Device;

use crate::gear_indicator::GearIndicator;
use crate::lap_bar::LapBar;

#[derive(Default, Debug, PartialEq)]
enum AppStatus {
    Running,
    Crashed,
    #[default]
    Connecting,
    FailedToConnect,
}

#[derive(Default)]
pub struct Data {
    lap_info: Option<RTLapInfo>,
    car_info: Option<RTCarInfo>,
    status: AppStatus,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum DataChannel {
    AppState,
    CarInfo,
    LapInfo,
}

impl RadioChannel<Data> for DataChannel {}

fn main() {
    let mut radio_station = RadioStation::create_global(Data::default());

    // A stand in for checking that we receive data.
    let data_queue = Arc::new(crossbeam::queue::ArrayQueue::<ACEvent>::new(2));
    let (mut status_tx, mut status_rx) = futures_channel::mpsc::channel::<AppStatus>(1);

    // queue that goes to the spawned thread
    let thread_queue = data_queue.clone();

    let launch_config = LaunchConfig::new()
        .with_future(move |_| async move {
            std::thread::spawn(move || {
                let Ok(client) = Client::new("0.0.0.0:0", Device::IPhone)
                    .inspect_err(|why| eprintln!("{why:?}"))
                else {
                    let _ = status_tx
                        .try_send(AppStatus::FailedToConnect)
                        .inspect_err(|why| eprintln!("{why:?}"));
                    return;
                };

                if let Err(why) = client.send_handshake() {
                    eprintln!("{why:?}");

                    let _ = status_tx
                        .try_send(AppStatus::FailedToConnect)
                        .inspect_err(|why| eprintln!("{why:?}"));
                }
                if let Err(why) = status_tx.try_send(AppStatus::Running) {
                    eprintln!("Unable to set Running status: {why:?}");

                    let _ = status_tx
                        .try_send(AppStatus::Crashed)
                        .inspect_err(|why| eprintln!("failed to send app status: {why:?}"));
                    return;
                }

                while let Ok(msg) = client.recv_raw_event_buffer() {
                    let _ = thread_queue.force_push(msg);
                }
            });

            // LOOP: bounded by receiving a crashed event.

            loop {
                if let Ok(status) = status_rx.recv().await
                    && (status == AppStatus::FailedToConnect || status == AppStatus::Crashed)
                {
                    radio_station.write_channel(DataChannel::AppState).status = status;
                    return;
                }

                if let Some(msg) = data_queue.pop() {
                    match msg {
                        ACEvent::CarInfo(car_info) => {
                            radio_station.write_channel(DataChannel::CarInfo).car_info =
                                Some(*car_info);
                        }
                        ACEvent::LapInfo(lap_info) => {
                            radio_station.write_channel(DataChannel::LapInfo).lap_info =
                                Some(*lap_info);
                        }
                        _ => continue,
                    }
                }
            }
        })
        .with_window(WindowConfig::new_app(MyApp { radio_station }));

    launch(launch_config);
}

struct MyApp {
    radio_station: RadioStation<Data, DataChannel>,
}
impl App for MyApp {
    fn render(&self) -> impl IntoElement {
        use_share_radio(move || self.radio_station);

        rect()
            .expanded()
            .center()
            .spacing(6.0)
            .child(LapBar)
            .child(AppStatusComponent)
            .child(GearIndicator)
    }
}

#[derive(PartialEq)]
struct AppStatusComponent;

impl Component for AppStatusComponent {
    fn render(&self) -> impl IntoElement {
        let app_state_radio = use_radio(DataChannel::AppState);
        rect()
            .position(Position::new_global().top(0.0))
            .child(label().text(format!("{:?}", app_state_radio.read().status)))
    }
}
