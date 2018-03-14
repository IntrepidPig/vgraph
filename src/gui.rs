use std::sync::{Arc, Mutex};

use gtk::{self, Window, WindowType, Inhibit, TextView, Button, Box, Orientation, TextBuffer};
use gtk::{ButtonExt, WidgetExt, ContainerExt, TextViewExt, TextBufferExt};
use relm::{Widget, Update, Relm};

pub struct Model {
	new_eqns: Arc<Mutex<Option<Vec<String>>>>,
}

pub struct Win {
	model: Model,
	window: Window,
	entry: TextView,
}

impl Update for Win {
	type Model = Model;
	type ModelParam = Arc<Mutex<Option<Vec<String>>>>;
	type Msg = Msg;
	
	fn model(_: &Relm<Self>, param: Self::ModelParam) -> Self::Model {
		Self::Model {
			new_eqns: param,
		}
	}
	
	fn update(&mut self, event: Msg) {
		match event {
			Msg::Apply => {
				let mut eqns = Vec::new();
				let buf = self.entry.get_buffer().unwrap();
				let (start, end) = buf.get_bounds();
				let raw = buf.get_text(&start, &end, false).unwrap();
				for line in raw.lines() {
					eqns.push(line.to_string());
				}
				*self.model.new_eqns.lock().unwrap() = Some(eqns);
			},
			Msg::Quit => {
				gtk::main_quit();
			}
		}
	}
}

impl Widget for Win {
	type Root = Window;
	
	fn root(&self) -> Self::Root {
		self.window.clone()
	}
	
	fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
		let window = Window::new(WindowType::Toplevel);
		let container = Box::new(Orientation::Vertical, 6);
		let entry = TextView::new();
		let apply = Button::new_with_label("Apply");
		
		container.add(&entry);
		container.add(&apply);
		window.add(&container);
		
		window.show_all();
		
		connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));
		connect!(relm, apply, connect_clicked(_), Msg::Apply);
		
		Win {
			model,
			window,
			entry
		}
	}
}

#[derive(Debug, Msg)]
pub enum Msg {
	Apply,
	Quit,
}
