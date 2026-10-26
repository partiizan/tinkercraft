//! Depth-tested native OpenGL viewport. Geometry buffers are reused while orbiting.
use eframe::{
    egui,
    glow::{self, HasContext},
};
use glam::{DMat4, DVec3};
use std::sync::{Arc, Mutex};
use tinkercraft_core::{
    geometry::SourceMesh, operations::Selection, spatial::Workplane, viewport::Camera,
};
pub struct Renderer {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    vertices: Vec<f32>,
    triangles: i32,
    total: i32,
    dirty: bool,
    pub key: String,
}
impl Renderer {
    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            let program = gl.create_program()?;
            let mut shaders = vec![];
            for (kind, source) in [
                (
                    glow::VERTEX_SHADER,
                    "#version 330 core\nlayout(location=0) in vec3 position;layout(location=1) in vec3 normal;layout(location=2) in vec4 color;uniform mat4 matrix;out vec3 n;out vec4 c;void main(){gl_Position=matrix*vec4(position,1.0);n=normal;c=color;}",
                ),
                (
                    glow::FRAGMENT_SHADER,
                    "#version 330 core\nin vec3 n;in vec4 c;out vec4 out_color;void main(){if(c.a<0.0 && mod(floor(gl_FragCoord.x)+floor(gl_FragCoord.y),7.0)>2.0)discard;float light=length(n)<0.1?1.0:0.42+0.58*abs(dot(normalize(n),normalize(vec3(-0.4,-0.6,1.0))));out_color=vec4(c.rgb*light,1.0);}",
                ),
            ] {
                let shader = gl.create_shader(kind)?;
                gl.shader_source(shader, source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    let e = gl.get_shader_info_log(shader);
                    gl.delete_shader(shader);
                    for s in shaders {
                        gl.delete_shader(s);
                    }
                    gl.delete_program(program);
                    return Err(e);
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }
            gl.link_program(program);
            let ok = gl.get_program_link_status(program);
            let error = gl.get_program_info_log(program);
            for s in shaders {
                gl.detach_shader(program, s);
                gl.delete_shader(s);
            }
            if !ok {
                gl.delete_program(program);
                return Err(error);
            }
            let vao = gl.create_vertex_array()?;
            let vbo = gl.create_buffer()?;
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            for (i, size, offset) in [(0, 3, 0), (1, 3, 12), (2, 4, 24)] {
                gl.enable_vertex_attrib_array(i);
                gl.vertex_attrib_pointer_f32(i, size, glow::FLOAT, false, 40, offset);
            }
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            Ok(Self {
                program,
                vao,
                vbo,
                vertices: vec![],
                triangles: 0,
                total: 0,
                dirty: true,
                key: String::new(),
            })
        }
    }
    pub fn prepare(
        &mut self,
        key: String,
        sources: &[SourceMesh],
        selection: &Selection,
        plane: &Workplane,
        dark: bool,
    ) {
        if self.key == key {
            return;
        }
        self.key = key;
        self.vertices.clear();
        for s in sources {
            let color = if s.hole {
                [232, 178, 106]
            } else if selection.contains(&s.id) {
                s.color.map(|v| (v as u16 + 35).min(255) as u8)
            } else {
                s.color
            };
            for t in &s.mesh.triangles {
                let v = t.map(|i| DVec3::from(s.mesh.vertices[i]));
                let normal = (v[1] - v[0]).cross(v[2] - v[0]).normalize_or_zero();
                for p in v {
                    self.vertices.extend(p.to_array().map(|v| v as f32));
                    self.vertices.extend(normal.to_array().map(|v| v as f32));
                    self.vertices.extend(color.map(|v| v as f32 / 255.));
                    self.vertices.push(if s.hole { -1. } else { 1. });
                }
            }
        }
        self.triangles = (self.vertices.len() / 10) as i32;
        let color = if dark {
            [0.25, 0.3, 0.34]
        } else {
            [0.7, 0.75, 0.8]
        };
        for i in -100..=100 {
            for axis in 0..2 {
                for end in [-1000., 1000.] {
                    let p = if axis == 0 {
                        DVec3::new(i as f64 * 10., end, -0.04)
                    } else {
                        DVec3::new(end, i as f64 * 10., -0.04)
                    };
                    self.vertices
                        .extend(plane.world(p).to_array().map(|v| v as f32));
                    self.vertices.extend([0.; 3]);
                    self.vertices.extend(color);
                    self.vertices.push(1.);
                }
            }
        }
        self.total = (self.vertices.len() / 10) as i32;
        self.dirty = true;
    }
    pub fn callback(
        renderer: Arc<Mutex<Self>>,
        camera: &Camera,
        rect: egui::Rect,
    ) -> egui::PaintCallback {
        let aspect = rect.width() as f64 / rect.height().max(1.) as f64;
        let half = camera.distance * 19_f64.to_radians().tan();
        let projection = if camera.orthographic {
            DMat4::orthographic_rh_gl(-half * aspect, half * aspect, -half, half, 0.01, 100000.)
        } else {
            DMat4::perspective_rh_gl(38_f64.to_radians(), aspect, 0.01, 100000.)
        };
        let matrix = (projection
            * DMat4::look_at_rh(camera.eye(), camera.target, camera.basis().1))
        .as_mat4()
        .to_cols_array();
        egui::PaintCallback {
            rect,
            callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |_info, painter| {
                if let Ok(mut r) = renderer.lock() {
                    unsafe {
                        let gl = painter.gl();
                        gl.enable(glow::DEPTH_TEST);
                        gl.depth_func(glow::LEQUAL);
                        gl.depth_mask(true);
                        gl.clear_depth_f32(1.);
                        gl.clear(glow::DEPTH_BUFFER_BIT);
                        gl.disable(glow::CULL_FACE);
                        gl.disable(glow::BLEND);
                        gl.use_program(Some(r.program));
                        gl.bind_vertex_array(Some(r.vao));
                        gl.bind_buffer(glow::ARRAY_BUFFER, Some(r.vbo));
                        if r.dirty {
                            let bytes = std::slice::from_raw_parts(
                                r.vertices.as_ptr().cast::<u8>(),
                                r.vertices.len() * 4,
                            );
                            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::DYNAMIC_DRAW);
                            r.dirty = false;
                        }
                        gl.uniform_matrix_4_f32_slice(
                            gl.get_uniform_location(r.program, "matrix").as_ref(),
                            false,
                            &matrix,
                        );
                        gl.draw_arrays(glow::TRIANGLES, 0, r.triangles);
                        gl.draw_arrays(glow::LINES, r.triangles, r.total - r.triangles);
                        gl.bind_vertex_array(None);
                        gl.bind_buffer(glow::ARRAY_BUFFER, None);
                        gl.use_program(None);
                        gl.disable(glow::DEPTH_TEST);
                        gl.enable(glow::BLEND);
                    }
                }
            })),
        }
    }
    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
            gl.delete_program(self.program);
        }
    }
}
