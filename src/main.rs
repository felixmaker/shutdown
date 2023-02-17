#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fltk::{prelude::*, *};
use system_shutdown::shutdown;
use tokio::task::JoinHandle;

#[derive(Clone, Copy)]
enum Message {
    Schedule,
    Cancel,
    Update(std::time::Duration)
}

#[tokio::main]
async fn main() {
    
    let app = app::App::default();
    let mut main_window = window::Window::new(100, 100, 280, 130, "Shutdown Scheduler");

    let (s, r) = app::channel();
    
    let mut vpack = group::Pack::new(0, 0, 260, 100, None)
        .center_of_parent();

    vpack.set_spacing(10);

    frame::Frame::new(0, 0, 260, 20, "Shutdown computer in: ")
        .set_align(enums::Align::Left | enums::Align::Inside);

    let mut duration_input = input::Input::new(0, 0, 260, 30, None);
    duration_input.set_value("5min");
    
    let mut hpack = group::Pack::new(0, 0, 260, 30, None)
        .with_type(group::PackType::Horizontal);    
    hpack.set_spacing(10);
    let mut schedule_button = button::Button::new(0, 0, 125, 30, "Schedule");
    schedule_button.emit(s, Message::Schedule);
    button::Button::new(0, 0, 125, 30, "Cancel").emit(s, Message::Cancel);
    hpack.end();

    vpack.end();

    main_window.end();
    main_window.show();

    let mut jobs: Vec<JoinHandle<()>> = Vec::new();

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::Schedule => {
                    let duration_input_value = duration_input.value();
                    if let Ok(duration) = humantime::parse_duration(&duration_input_value) {   
                        let end_time = tokio::time::Instant::now() + duration.into();
                        let times = duration.as_secs() / 1 + 1;
                        duration_input.deactivate();
                        schedule_button.deactivate();
                        jobs.push(tokio::spawn(async move {
                            if let Err(_) = tokio::time::timeout_at(end_time, async {
                                let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                                for _ in 0..times {
                                    interval.tick().await;
                                    let time_left = end_time - tokio::time::Instant::now();
                                    s.send(Message::Update(time_left))
                                }
                                shutdown().unwrap();                                
                            }).await {
                                shutdown().unwrap();
                            }
                        }));
                    }
                }
                Message::Cancel => {
                    for job in &jobs {
                        job.abort();
                    }
                    schedule_button.activate();
                    duration_input.activate();

                }
                Message::Update(time_left) => {                    
                    let human_time = format!("{}", humantime::format_duration(time_left));
                    duration_input.set_value(&human_time);
                }
            }
        }
    }
}