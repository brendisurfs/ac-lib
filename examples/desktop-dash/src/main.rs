use std::sync::Arc;

use freya::{prelude::*, radio::*};

use ac_lib::Client;
use ac_lib::parser::Device;

#[derive(Default, Debug)]
enum AppStatus {
    #[default]
    Running,
    Crashed,
    Connecting,
    FailedToConnect,
}

pub struct Data {
    stream_data: [u8; 512],
    status: AppStatus,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            stream_data: [0; 512],
            status: AppStatus::Connecting,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum DataChannel {
    AppState,
    Data,
}

impl RadioChannel<Data> for DataChannel {}

fn main() {
    let mut radio_station = RadioStation::create_global(Data::default());

    // A stand in for checking that we receive data.
    let data_queue = Arc::new(crossbeam::queue::ArrayQueue::<[u8; 512]>::new(2));
    let (mut status_tx, mut status_rx) = futures_channel::mpsc::channel::<AppStatus>(1);

    // queue that goes to the spawned thread
    let thread_queue = data_queue.clone();

    let launch_config = LaunchConfig::new()
        .with_future(move |_| async move {
            std::thread::spawn(move || {
                let Ok(client) = Client::new("0.0.0.0:0", Device::IPhone) else {
                    let _ = status_tx
                        .try_send(AppStatus::FailedToConnect)
                        .inspect_err(|why| eprintln!("{why:?}"));
                    return;
                };

                client.send_handshake().expect("failed to send handshake");

                while let Ok(msg) = client.recv_raw_event_buffer() {
                    let _ = thread_queue.force_push(msg.1);
                }
            });

            // LOOP: bounded by receiving a crashed event.

            loop {
                if let Ok(AppStatus::Crashed) = status_rx.recv().await {
                    radio_station.write_channel(DataChannel::AppState).status = AppStatus::Crashed;
                    return;
                }
                if let Some(msg) = data_queue.pop() {
                    radio_station.write_channel(DataChannel::Data).stream_data = msg;
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

        let radio = use_radio(DataChannel::AppState);

        rect()
            .expanded()
            .center()
            .spacing(6.0)
            .child(format!("App Status: {:?}", radio.read().status))
    }
}

#[derive(PartialEq)]
struct GearIndicator(i32);

impl Component for GearIndicator {
    fn render(&self) -> impl IntoElement {
        rect().border(None)
    }
}

#[derive(PartialEq)]
struct CoolComponent(i32);
impl Component for CoolComponent {
    fn render(&self) -> impl IntoElement {
        let mut state = use_state(|| self.0);

        let increase = move |_| {
            *state.write() += 1;
        };

        Button::new()
            .on_press(increase)
            .child(format!("Value: {}", state.read()))
    }
}
