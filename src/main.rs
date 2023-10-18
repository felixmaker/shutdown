// Copyright (c) 2023, felixmaker
// SPDX-License-Identifier: MIT

use std::rc::Rc;

use chrono::prelude::*;
use slint::{Timer, TimerMode};

fn caculate_future_time(seconds: i32) -> String {
    let local: DateTime<Local> = Local::now();
    let future = local + chrono::Duration::seconds(seconds as i64);
    future.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn main() {
    let main_window = MainWindow::new().unwrap();
    let timer = Rc::new(Timer::default());

    main_window.on_caculate_future_time(|seconds| caculate_future_time(seconds).into());

    main_window.on_send_shutdown_signal({
        let timer = timer.clone();
        move |seconds| {
            timer.stop();
            timer.start(
                TimerMode::SingleShot,
                std::time::Duration::from_secs(seconds as u64),
                move || {
                    let _ = system_shutdown::shutdown();
                },
            );
        }
    });

    main_window.on_cancal_shutdown_signal({
        let timer = timer.clone();
        move || {
            timer.stop();
        }
    });

    main_window.run().unwrap();
}

slint::slint! {
    import { HorizontalBox , SpinBox, VerticalBox, Button, Slider} from "std-widgets.slint";
    export component MainWindow inherits Window {
        default-font-size: 14px;
        min-width: 300px;
        min-height: 200px;
        max-height: 250px;
        title: @tr("Shutdown Scheduler");

        callback send-shutdown-signal(int);
        callback cancal-shutdown-signal();
        callback caculate_future_time(int) -> string;

        property <bool> commit-enabled: true;
        property <string> status: @tr("Unplanned");

        in-out property <float> hours;
        in-out property <float> minutes;
        in-out property <float> seconds;

        function total_seconds() -> int {
            return root.hours * 3600 + root.minutes * 60 + root.seconds;
        }

        property <length> slider-width: 200px;
        property <length> slider-height: 20px;

        VerticalBox {
            padding: 15px;
            spacing: 10px;

            Text {
                height: 20px;
                text: @tr("Send shutdown signal in...");
            }

            GridLayout {
                spacing: 8px;

                Row {
                    Text { text: @tr("Hours: ");}
                    Slider {
                        maximum: 24;
                        value <=> root.hours;
                    }
                    Text {
                        width: 20px;
                        height: 20px;
                        text: Math.ceil(root.hours);
                    }
                }

                Row {
                    Text { text: @tr("Minutes: ");}
                    Slider {
                        maximum: 60;
                        value <=> root.minutes;
                    }
                    Text {
                        width: 20px;
                        height: 20px;
                        text: Math.ceil(root.minutes);
                    }
                }

                Row {
                    Text { text: @tr("Seconds: ");}
                    Slider {
                        maximum: 60;
                        value <=> root.seconds;
                    }
                    Text {
                        width: 20px;
                        height: 20px;
                        text: Math.ceil(root.seconds);
                    }
                }
            }

            HorizontalBox {
                padding: 0px;

                Button {
                    text: @tr("Commit");
                    height: 30px;
                    primary: true;
                    enabled <=> root.commit-enabled;

                    clicked => {
                        root.send-shutdown-signal(root.total_seconds());
                        root.commit-enabled = false;
                        root.status = root.caculate-future-time(root.total_seconds());
                    }
                }
                Button { text: @tr("Reset"); height: 30px;
                    clicked => {
                        root.cancal-shutdown-signal();
                        root.hours = 0;
                        root.minutes = 0;
                        root.seconds = 0;
                        root.commit-enabled = true;
                        root.status = @tr("Unplanned");
                    }
                }
            }

            Text {
                text: @tr("Shutdown time: {}", root.status);
            }
        }

    }
}
