use mexprp::{Expr, Context};
use vrender::td::{Vertex, Color};
use vrender::render::Render;

pub struct Graph {
	eqn: Expr,
	steps: u32,
	range: f64,
	verts: Option<Vec<Vertex>>,
	indcs: Option<Vec<u32>>,
}

impl Graph {
	pub fn new(eqn: Expr, steps: u32, range: f64) -> Self {
		let mut graph = Graph {
			eqn,
			steps,
			range,
			verts: None,
			indcs: None,
		};
		graph.graph();
		graph
	}
	
	pub fn graph(&mut self) {
		fn calc(x: f64, z: f64, expr: &Expr) -> Vertex {
			let mut ctx = Context::new();
			ctx.add("x", x as f64);
			ctx.add("z", z as f64);
			let y = expr.eval_ctx(&ctx).unwrap() as f32;
			let x = x as f32;
			let z = z as f32;
			Vertex::new(x, -y, z, 1.0, Color::green())
		}
		
		let mut verts = Vec::with_capacity(self.steps as usize * self.steps as usize * 4);
		let mut indices = Vec::with_capacity(self.steps as usize * self.steps as usize * 6);
		let mut index = 0;
		let mut x = -self.range;
		let mut z = -self.range;
		while x < self.range {
			while z < self.range {
				verts.push(calc(x, z, &self.eqn));
				verts.push(calc(x + 1.0, z, &self.eqn));
				verts.push(calc(x, z + 1.0, &self.eqn));
				verts.push(calc(x + 1.0, z + 1.0, &self.eqn));
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
		self.verts = Some(verts);
		self.indcs = Some(indices)
	}
}

impl Render for Graph {
	fn vbuf(&self) -> &[Vertex] {
		if self.verts.is_some() {
			self.verts.as_ref().unwrap().as_slice()
		} else {
			&[]
		}
	}
	
	fn ibuf(&self) -> &[u32] {
		if self.indcs.is_some() {
			self.indcs.as_ref().unwrap().as_slice()
		} else {
			&[]
		}
	}
}