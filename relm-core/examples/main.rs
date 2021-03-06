/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

extern crate chrono;
extern crate futures;
extern crate glib;
extern crate glib_itc;
extern crate gtk;
extern crate relm_core;
extern crate tokio_core;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Local;
use futures::Stream;
use glib::Continue;
use glib_itc::channel;
use gtk::{Button, ButtonExt, ContainerExt, Inhibit, Label, WidgetExt, Window, WindowType};
use gtk::Orientation::Vertical;
use relm_core::{Core, EventStream};
use tokio_core::reactor::Interval;

use self::Msg::*;

struct Widgets {
    clock_label: Label,
    counter_label: Label,
}

#[derive(Clone, Debug)]
enum Msg {
    Clock,
    Decrement,
    Increment,
    Quit,
}

struct Model {
    counter: i32,
}

fn main() {
    gtk::init().unwrap();

    let vbox = gtk::Box::new(Vertical, 0);

    let clock_label = Label::new(None);
    vbox.add(&clock_label);

    let plus_button = Button::new_with_label("+");
    vbox.add(&plus_button);

    let counter_label = Label::new("0");
    vbox.add(&counter_label);

    let widgets = Widgets {
        clock_label: clock_label,
        counter_label: counter_label,
    };

    let window = Window::new(WindowType::Toplevel);
    window.add(&vbox);

    let (sender, mut receiver) = channel();
    let sender = Arc::new(Mutex::new(sender));

    let stream = EventStream::new(sender.clone());

    let other_widget_stream = EventStream::new(sender);
    {
        stream.observe(move |event: Msg| {
            other_widget_stream.emit(Quit);
            println!("Event: {:?}", event);
        });
    }

    {
        let stream = stream.clone();
        plus_button.connect_clicked(move |_| {
            stream.emit(Increment);
        });
    }

    let minus_button = Button::new_with_label("-");
    vbox.add(&minus_button);
    {
        let stream = stream.clone();
        minus_button.connect_clicked(move |_| {
            stream.emit(Decrement);
        });
    }

    window.show_all();

    {
        let stream = stream.clone();
        window.connect_delete_event(move |_, _| {
            stream.emit(Quit);
            Inhibit(false)
        });
    }

    let mut model = Model {
        counter: 0,
    };

    fn update(event: Msg, model: &mut Model, widgets: &Widgets) {
        match event {
            Clock => {
                let now = Local::now();
                widgets.clock_label.set_text(&now.format("%H:%M:%S").to_string());
            },
            Decrement => {
                model.counter -= 1;
                widgets.counter_label.set_text(&model.counter.to_string());
            },
            Increment => {
                model.counter += 1;
                widgets.counter_label.set_text(&model.counter.to_string());
            },
            Quit => gtk::main_quit(),
        }
    }

    {
        let stream = stream.clone();
        receiver.connect_recv(move || {
            if let Some(event) = stream.pop_ui_events() {
                update(event, &mut model, &widgets);
            }
            Continue(true)
        });
    }

    let stream = stream.clone();
    let remote = Core::run();
    remote.spawn(move |handle| {
        let interval = {
            let interval = Interval::new(Duration::from_secs(1), handle).unwrap();
            let stream = stream.clone();
            interval.map_err(|_| ()).for_each(move |_| {
                stream.emit(Clock);
                Ok(())
            })
        };
        handle.spawn(interval);

        let event_future = stream.for_each(|_| Ok(()));

        handle.spawn(event_future);

        Ok(())
    });
    gtk::main();
}
