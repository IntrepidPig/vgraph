extern crate mexprp;
extern crate vrender;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, Receiver};

use relm::Widget;

use mexprp::Expr;
use vrender::{App, Renderer, Context};
use vrender::obj::{Object, Mesh};
use vrender::td::{Camera, Vec3};
use vrender::math::{PerspectiveFov, Deg, Euler, InnerSpace, Zero};
use vrender::window::{self, Event, CursorState, MouseCursor, MouseButton};

mod td;
mod gui;

use td::Graph;

struct Player {
	pub camera: Camera,
	pub speed: f32,
}

impl Player {
	pub fn walk(&mut self, mov: Vec3) {
		let old_pos = self.camera.get_pos();
		let vecs = self.camera.get_vec();
		let (front, right) = (vecs.0, vecs.1);
		let (forward, upward, sideward) = (mov.z, mov.y, mov.x);
		self.camera.set_pos(old_pos + Vec3::new(front.x, 0.0, front.z).normalize() * (forward * self.speed));
		let old_pos = self.camera.get_pos();
		self.camera.set_pos(old_pos + Vec3::new(right.x, 0.0, right.z).normalize() * (sideward * self.speed));
		let old_pos = self.camera.get_pos();
		self.camera.set_pos(Vec3::new(old_pos.x, old_pos.y + upward * self.speed, old_pos.z))
	}
	
	pub fn spin(&mut self, amt: Deg<f32>) {
		let old_rot = self.camera.get_rot();
		self.camera.set_rot(Euler::new(old_rot.x, (old_rot.y + amt * 15.0) % Deg(360.0), old_rot.z));
	}
	
	pub fn crane(&mut self, amt: Deg<f32>) {
		let old_rot = self.camera.get_rot();
		let mut rot: Deg<f32> = old_rot.x + amt * 15.0;
		if rot.0.partial_cmp(&89.99).unwrap() == std::cmp::Ordering::Greater {
			rot.0 = 89.99;
		} else if rot.0.partial_cmp(&-89.99).unwrap() == std::cmp::Ordering::Less {
			rot.0 = -89.99;
		}
		self.camera.set_rot(Euler::new(rot, old_rot.y, old_rot.z));
	}
}

struct Grapher {
	eqn_recvr: Receiver<Vec<String>>,
	player: Player,
	running: bool,
	move_z: (bool, bool),
	move_x: (bool, bool),
	move_y: (bool, bool),
	captured: bool,
	range: f64,
	steps: u32,
	time: f32,
}

impl Grapher {
	pub fn new(eqn_recvr: Receiver<Vec<String>>) -> Self {
		Grapher {
			eqn_recvr,
			player: Player {
				camera: Camera::new(PerspectiveFov {
					fovy: Deg(90.0).into(),
					aspect: 1.0,
					near: 0.1,
					far: 1000.0,
				}),
				speed: 3.5,
			},
			running: true,
			move_z: (false, false),
			move_x: (false, false),
			move_y: (false, false),
			captured: false,
			range: 32f64,
			steps: 256u32,
			time: 0f32,
		}
	}
}

impl App for Grapher {
	fn get_camera(&mut self) -> &mut Camera {
		&mut self.player.camera
	}
	
	fn handle_event(&mut self, event: Event, context: Context) {
		use window::Event;
		use window::DeviceEvent;
		use window::WindowEvent;
		use window::KeyboardInput;
		use window::VirtualKeyCode::*;
		use window::ElementState;
		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::KeyboardInput {
					device_id: _, input: KeyboardInput { scancode: _, state: ElementState::Released, virtual_keycode: Some(Escape), modifiers: _ }
				} => {
					self.captured = false;
					context.window.set_cursor_state(CursorState::Normal).unwrap();
					context.window.set_cursor(MouseCursor::Default);
				},
				WindowEvent::Closed => {
					self.running = false;
				},
				WindowEvent::KeyboardInput {
					device_id: _, input: KeyboardInput { scancode: _, state, virtual_keycode: key, modifiers: _mods }
				} => {
					match state {
						ElementState::Pressed => {
							match key {
								Some(W) => self.move_z.0 = true,
								Some(S) => self.move_z.1 = true,
								Some(A) => self.move_x.0 = true,
								Some(D) => self.move_x.1 = true,
								Some(Space) => self.move_y.0 = true,
								Some(LShift) => self.move_y.1 = true,
								_ => {},
							}
						},
						ElementState::Released => {
							match key {
								Some(W) => self.move_z.0 = false,
								Some(S) => self.move_z.1 = false,
								Some(A) => self.move_x.0 = false,
								Some(D) => self.move_x.1 = false,
								Some(Space) => self.move_y.0 = false,
								Some(LShift) => self.move_y.1 = false,
								_ => {},
							}
						}
					}
				},
				WindowEvent::MouseInput {
					device_id: _, state, button, modifiers: _
				} => {
					if button == MouseButton::Left && state == ElementState::Pressed {
						self.captured = true;
						context.window.set_cursor_state(CursorState::Grab).unwrap();
						context.window.set_cursor(MouseCursor::NoneCursor);
					}
				},
				_ => {}
			},
			Event::DeviceEvent { event, .. } => match event {
				DeviceEvent::Motion { axis, value } => {
					if self.captured {
						match axis {
							0 => {
								self.player.spin(Deg((value / 200.0f64) as f32));
							},
							1 => {
								self.player.crane(Deg((value / 200.0f64) as f32));
							},
							_ => {}
						}
					}
				},
				_ => {}
			},
			_ => {}
		}
	}
	
	fn update(&mut self, ms: f32, context: Context) {
		let mut movement: Vec3 = Vec3::zero();
		if self.move_x.0 { movement.x -= 1.0 };
		if self.move_x.1 { movement.x += 1.0 };
		if self.move_y.0 { movement.y -= 1.0 };
		if self.move_y.1 { movement.y += 1.0 };
		if self.move_z.0 { movement.z += 1.0 };
		if self.move_z.1 { movement.z -= 1.0 };
		
		if self.captured {
			self.player.walk(movement * ms / 200.0);
		}
		
		if let Ok(new_eqns) = self.eqn_recvr.try_recv() {
			context.targets.clear();
			for new_eqn in &new_eqns {
				match Expr::from(&new_eqn) {
					Ok(expr) => {
						let graph = Graph::new(expr, self.steps, self.range);
						let mesh = Mesh::new(context.internal, graph.vbuf().to_vec(), graph.ibuf().to_vec()).unwrap();
						context.targets.insert(new_eqn.to_string(), Object::from_mesh(mesh));
					},
					Err(e) => {
						println!("{}", e);
					}
				}
			}
		}
		
		self.time += ms;
	}
	
	fn is_running(&self) -> bool {
		self.running
	}
}

fn main() {
	let (tx, rx) = mpsc::channel::<Vec<String>>();
	
	let handle = thread::spawn(|| {
		thread::sleep(Duration::new(2, 0)); // Awful hack
		let app = Grapher::new(rx);
		let mut renderer = Renderer::new(app);
		renderer.run();
	});
	
	gui::Win::run(tx).unwrap();
	
	handle.join().unwrap();
}
