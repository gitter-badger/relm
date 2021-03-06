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
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate simplelog;
extern crate tokio_core;

use std::time::Duration;

use chrono::Local;
use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    Dialog,
    DialogExt,
    Inhibit,
    Label,
    WidgetExt,
    Window,
    WindowType,
    DIALOG_MODAL,
};
use gtk::Orientation::Vertical;
use relm::{Relm, RemoteRelm, Widget};
use simplelog::{Config, TermLogger};
use simplelog::LogLevelFilter::Warn;
use tokio_core::reactor::Interval;

use self::Msg::*;

#[derive(Msg)]
enum Msg {
    Open(i32),
    Quit,
    Tick(()),
}

#[derive(Clone)]
struct Win {
    label: Label,
    num_label: Label,
    window: Window,
}

impl Widget for Win {
    type Model = ();
    type ModelParam = ();
    type Msg = Msg;
    type Root = Window;

    fn model(_: ()) -> () {
        ()
    }

    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn subscriptions(relm: &Relm<Msg>) {
        let stream = Interval::new(Duration::from_secs(1), relm.handle()).unwrap();
        relm.connect_exec_ignore_err(stream, Tick);
    }

    fn update(&mut self, event: Msg, _model: &mut ()) {
        match event {
            Open(num) => {
                self.num_label.set_text(&num.to_string());
            },
            Tick(()) => {
                let time = Local::now();
                self.label.set_text(&format!("{}", time.format("%H:%M:%S")));
            },
            Quit => gtk::main_quit(),
        }
    }

    fn view(relm: &RemoteRelm<Self>, _model: &Self::Model) -> Self {
        let button = Button::new_with_label("Open");
        let label = Label::new(None);
        let num_label = Label::new(None);

        let window = Window::new(WindowType::Toplevel);

        let vbox = gtk::Box::new(Vertical, 0);
        vbox.add(&label);
        vbox.add(&button);
        vbox.add(&num_label);

        window.add(&vbox);

        window.show_all();

        {
            let window = window.clone();
            connect!(relm, button, connect_clicked(_), {
                let num = dialog(&window);
                Open(num)
            });
        }
        connect!(relm, window, connect_delete_event(_, _) (Some(Quit), Inhibit(false)));

        let mut win = Win {
            label: label,
            num_label: num_label,
            window: window,
        };
        win.update(Tick(()), &mut ());
        win
    }
}

fn dialog(window: &Window) -> i32 {
    let buttons = &[("Yes", 1), ("No", 2)];
    let dialog = Dialog::new_with_buttons(Some("Dialog"), Some(window), DIALOG_MODAL, buttons);
    let result = dialog.run();
    dialog.destroy();
    result
}

fn main() {
    TermLogger::init(Warn, Config::default()).unwrap();
    Win::run(()).unwrap();
}
