extern crate mexprp;
extern crate vrender;

use mexprp::{Expr, Context};
use vrender::{App};
use vrender::td::{Vertex, Camera, Color, Vec3};
use vrender::math::{PerspectiveFov, Deg, Euler, InnerSpace, Zero};
use vrender::window::{self, Event, WindowEvent, DeviceEvent};

static DATA: ([Vertex; 8], [u32; 36]) = (
	[
		Vertex { a_Pos: [-0.25, -0.25, -0.25, 1.0], a_Color: [0.0, 0.0, 1.0, 1.0] },
		Vertex { a_Pos: [ 0.25, -0.25, -0.25, 1.0], a_Color: [0.0, 0.0, 1.0, 1.0] },
		Vertex { a_Pos: [ 0.25, -0.25,  0.25, 1.0], a_Color: [0.0, 0.0, 1.0, 1.0] },
		Vertex { a_Pos: [-0.25, -0.25,  0.25, 1.0], a_Color: [0.0, 0.0, 1.0, 1.0] },
		Vertex { a_Pos: [-0.25,  0.25, -0.25, 1.0], a_Color: [0.0, 0.0, 1.0, 1.0] },
		Vertex { a_Pos: [ 0.25,  0.25, -0.25, 1.0], a_Color: [0.0, 0.0, 1.0, 1.0] },
		Vertex { a_Pos: [ 0.25,  0.25,  0.25, 1.0], a_Color: [0.0, 0.0, 1.0, 1.0] },
		Vertex { a_Pos: [-0.25,  0.25,  0.25, 1.0], a_Color: [0.0, 0.0, 1.0, 1.0] },
	],
	[
		0, 1, 2, 2, 3, 0, // top
		0, 1, 4, 4, 5, 1, // front
		1, 2, 5, 5, 6, 2, // right
		2, 3, 6, 6, 7, 3, // back
		3, 0, 7, 7, 4, 0, // left
		4, 5, 6, 6, 7, 4, // bottom
	]
);


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
	eqns: Vec<Expr>,
	player: Player,
	running: bool,
	data: Vec<(Vec<Vertex>, Vec<u32>)>,
	move_z: (bool, bool),
	move_x: (bool, bool),
	move_y: (bool, bool),
	get_eqn: bool,
	range: f64,
	steps: u32,
	time: f32,
}

impl Grapher {
	pub fn new() -> Grapher {
		Grapher {
			eqns: vec![Expr::from("(x * z) / 4").unwrap()],
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
			data: Vec::new(),
			move_z: (false, false),
			move_x: (false, false),
			move_y: (false, false),
			get_eqn: false,
			range: 8f64,
			steps: 16u32,
			time: 0f32,
		}
	}
	
	fn regen_data(&mut self) {
		fn calc(x: f64, z: f64, t: f32, expr: &Expr) -> Vertex {
			let mut ctx = Context::new();
			ctx.add("x", x as f64);
			ctx.add("z", z as f64);
			ctx.add("t", t as f64);
			let y = expr.eval_ctx(&ctx).unwrap() as f32;
			let x = x as f32;
			let z = z as f32;
			Vertex::new(x, -y, z, 1.0, Color::green())
		}
		self.data.clear();
		for expr in &self.eqns {
			let mut verts = Vec::with_capacity(self.steps as usize * self.steps as usize * 4);
			let mut indices = Vec::with_capacity(self.steps as usize * self.steps as usize * 6);
			let mut index = 0;
			let mut x = -self.range;
			let mut z = -self.range;
			while x < self.range {
				while z < self.range {
					verts.push(calc(x, z, self.time, &expr));
					verts.push(calc(x + 1.0, z, self.time, &expr));
					verts.push(calc(x, z + 1.0, self.time, &expr));
					verts.push(calc(x + 1.0, z + 1.0, self.time, &expr));
					indices.push(index * 4);
					indices.push(index * 4 + 1);
					indices.push(index * 4 + 2);
					indices.push(index * 4 + 2);
					indices.push(index * 4 + 3);
					indices.push(index * 4 + 1);
					index += 1;
					z += self.range * 2.0 / self.steps as f64;
				}
				z = -self.range;
				x += self.range * 2.0 / self.steps as f64;
			}
			self.data.push((verts, indices));
		}
		self.data.push((Vec::from(DATA.0.as_ref()), Vec::from(DATA.1.as_ref())));
	}
}

impl App for Grapher {
	fn get_data(&self) -> &Vec<(Vec<Vertex>, Vec<u32>)> {
		&self.data
	}
	
	fn get_camera(&mut self) -> &mut Camera {
		&mut self.player.camera
	}
	
	fn handle_event(&mut self, event: Event) {
		use window::Event;
		use window::DeviceEvent;
		use window::WindowEvent;
		use window::KeyboardInput;
		use window::VirtualKeyCode::*;
		use window::ElementState;
		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::KeyboardInput {
					device_id: _, input: KeyboardInput { scancode: _, state: _, virtual_keycode: Some(Escape), modifiers: _ }
				} | WindowEvent::Closed => {
					self.running = false;
				},
				WindowEvent::KeyboardInput {
					device_id: _, input: KeyboardInput { scancode: _, state, virtual_keycode: key, modifiers: mods }
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
								Some(Return) => self.get_eqn = true,
								Some(RShift) => self.eqns.clear(),
								_ => {},
							}
						}
					}
				},
				_ => {}
			},
			Event::DeviceEvent { event, .. } => match event {
				DeviceEvent::Motion { axis, value } => {
					match axis {
						0 => {
							self.player.spin(Deg((value / 200.0f64) as f32));
						},
						1 => {
							self.player.crane(Deg((value / 200.0f64) as f32));
						},
						_ => {}
					}
				},
				_ => {}
			},
			_ => {}
		}
	}
	
	fn update(&mut self, ms: f32) {
		self.regen_data();
		let mut movement: Vec3 = Vec3::zero();
		if self.move_x.0 { movement.x -= 1.0 };
		if self.move_x.1 { movement.x += 1.0 };
		if self.move_y.0 { movement.y -= 1.0 };
		if self.move_y.1 { movement.y += 1.0 };
		if self.move_z.0 { movement.z += 1.0 };
		if self.move_z.1 { movement.z -= 1.0 };
		
		self.player.walk(movement * ms / 200.0);
		
		if self.get_eqn {
			self.get_eqn = false;
			let raw = {
				let mut x = String::new();
				std::io::stdin().read_line(&mut x).expect("Failed to read input");
				x
			};
			match Expr::from(&raw) {
				Ok(expr) => {
					self.eqns.push(expr);
				},
				Err(e) => {
					println!("{}", e);
				}
			}
		}
		self.time += ms;
	}
	
	fn is_running(&mut self) -> &mut bool {
		&mut self.running
	}
}

fn main() {
	let mut app = Grapher::new();
	app.run();
}
