//! Portable native viewport: depth-tested CPU rasterizer, presented as an egui texture.
//! No browser, JavaScript, OS-specific graphics calls or geometry-engine dependencies.
use crate::geometry::{Bounds, Mesh, SourceMesh};
use glam::{DVec2, DVec3};
use std::collections::HashSet;
#[derive(Clone, Debug)]
pub struct Camera {
    pub target: DVec3,
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
    pub orthographic: bool,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            target: DVec3::new(40., 15., 2.5),
            yaw: -0.9,
            pitch: 0.65,
            distance: 160.,
            orthographic: false,
        }
    }
}
impl Camera {
    pub fn basis(&self) -> (DVec3, DVec3, DVec3) {
        let out = DVec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        );
        let right = DVec3::new(-self.yaw.sin(), self.yaw.cos(), 0.);
        let up = out.cross(right);
        (right, up, out)
    }
    pub fn fit(&mut self, b: Bounds, aspect: f64) {
        if b.min.is_finite() && b.max.is_finite() {
            self.target = b.center();
            self.distance = ((b.max - b.min).length() * 1.7 / aspect.min(1.)).max(40.);
        }
    }
    pub fn eye(&self) -> DVec3 {
        self.target + self.basis().2 * self.distance
    }
    pub fn units_per_pixel(&self, height: f64) -> f64 {
        2. * self.distance * (19f64.to_radians().tan()) / height
    }
    pub fn project(&self, p: DVec3, width: usize, height: usize) -> Option<[f64; 3]> {
        let (right, up, out) = self.basis();
        let d = p - self.eye();
        let z = -d.dot(out);
        if z <= 0.01 {
            return None;
        }
        let scale = height as f64
            / (2. * 19f64.to_radians().tan() * if self.orthographic { self.distance } else { z });
        Some([
            width as f64 / 2. + d.dot(right) * scale,
            height as f64 / 2. - d.dot(up) * scale,
            z,
        ])
    }
    pub fn ray(&self, p: DVec2, width: f64, height: f64) -> (DVec3, DVec3) {
        let (right, up, out) = self.basis();
        let offset = right * (p.x - width / 2.) * self.units_per_pixel(height)
            - up * (p.y - height / 2.) * self.units_per_pixel(height);
        if self.orthographic {
            (self.eye() + offset, -out)
        } else {
            (self.eye(), (self.target + offset - self.eye()).normalize())
        }
    }
    pub fn view(&mut self, name: &str) {
        match name {
            "Top" => {
                self.yaw = -std::f64::consts::FRAC_PI_2;
                self.pitch = std::f64::consts::FRAC_PI_2;
            }
            "Front" => {
                self.yaw = -std::f64::consts::FRAC_PI_2;
                self.pitch = 0.;
            }
            "Right" => {
                self.yaw = 0.;
                self.pitch = 0.;
            }
            _ => {
                self.yaw = -0.9;
                self.pitch = 0.65;
            }
        }
    }
}
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
    pub ids: Vec<u32>,
    pub names: Vec<String>,
}
impl Frame {
    pub fn pick(&self, x: usize, y: usize) -> Option<String> {
        if x < self.width && y < self.height {
            self.names
                .get(self.ids[y * self.width + x] as usize)
                .cloned()
        } else {
            None
        }
    }
}
pub fn render(
    camera: &Camera,
    width: usize,
    height: usize,
    sources: &[SourceMesh],
    selected: &HashSet<String>,
    result: Option<&Mesh>,
) -> Frame {
    render_themed(camera, width, height, sources, selected, result, false)
}
pub fn render_themed(
    camera: &Camera,
    width: usize,
    height: usize,
    sources: &[SourceMesh],
    selected: &HashSet<String>,
    result: Option<&Mesh>,
    dark: bool,
) -> Frame {
    render_on_plane(
        camera,
        width,
        height,
        sources,
        selected,
        result,
        dark,
        &crate::spatial::Workplane::default(),
    )
}
#[allow(clippy::too_many_arguments)]
pub fn render_on_plane(
    camera: &Camera,
    width: usize,
    height: usize,
    sources: &[SourceMesh],
    selected: &HashSet<String>,
    result: Option<&Mesh>,
    dark: bool,
    plane: &crate::spatial::Workplane,
) -> Frame {
    let mut frame = Frame {
        width,
        height,
        rgba: vec![0; width * height * 4],
        ids: vec![u32::MAX; width * height],
        names: vec![],
    };
    let mut depth = vec![f64::INFINITY; width * height];
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) * 4;
            let c = ((if dark { 39. } else { 235. }) - 12. * y as f64 / height as f64) as u8;
            frame.rgba[i..i + 4].copy_from_slice(&[
                c,
                c.saturating_add(5),
                c.saturating_add(7),
                255,
            ]);
        }
    }
    // Infinite XY workplane grid, intersected per pixel so it shares the depth buffer.
    for y in 0..height {
        for x in 0..width {
            let (o, d) = camera.ray(DVec2::new(x as f64, y as f64), width as f64, height as f64);
            let Some(p) = plane.intersect(o, d, 0.) else {
                continue;
            };
            let local = plane.local(p);
            let spacing = 10.;
            let near = |v: f64| ((v / spacing).round() * spacing - v).abs();
            let line = near(local.x).min(near(local.y));
            if line < camera.units_per_pixel(height as f64) * 0.45 {
                let i = y * width + x;
                frame.rgba[i * 4..i * 4 + 3].copy_from_slice(if dark {
                    &[65, 78, 87]
                } else {
                    &[186, 200, 207]
                });
                if let Some(q) = camera.project(p, width, height) {
                    depth[i] = q[2];
                }
            }
        }
    }
    fn draw(
        camera: &Camera,
        m: &Mesh,
        color: [u8; 3],
        id: Option<&str>,
        hole: bool,
        frame: &mut Frame,
        depth: &mut [f64],
    ) {
        let id_index = if let Some(id) = id {
            let i = frame.names.len() as u32;
            frame.names.push(id.to_owned());
            i
        } else {
            u32::MAX
        };
        let w = frame.width;
        let h = frame.height;
        let projected: Vec<_> = m
            .vertices
            .iter()
            .map(|p| camera.project(DVec3::from(*p), w, h))
            .collect();
        let light = DVec3::new(-0.4, -0.6, 1.).normalize();
        for t in &m.triangles {
            let [Some(a), Some(b), Some(c)] = t.map(|i| projected[i]) else {
                continue;
            };
            let edge = |u: [f64; 3], v: [f64; 3], x: f64, y: f64| {
                (x - u[0]) * (v[1] - u[1]) - (y - u[1]) * (v[0] - u[0])
            };
            let area = edge(a, b, c[0], c[1]);
            if area.abs() < 1e-8 {
                continue;
            }
            let minx = a[0].min(b[0]).min(c[0]).floor().max(0.) as usize;
            let maxx = a[0].max(b[0]).max(c[0]).ceil().min(w as f64 - 1.).max(0.) as usize;
            let miny = a[1].min(b[1]).min(c[1]).floor().max(0.) as usize;
            let maxy = a[1].max(b[1]).max(c[1]).ceil().min(h as f64 - 1.).max(0.) as usize;
            if minx >= w || miny >= h {
                continue;
            }
            let v = t.map(|i| DVec3::from(m.vertices[i]));
            let normal = (v[1] - v[0]).cross(v[2] - v[0]).normalize_or_zero();
            let shade = 0.48 + 0.52 * normal.dot(light).max(0.);
            let rgb = color.map(|v| (v as f64 * shade).min(255.) as u8);
            for y in miny..=maxy {
                for x in minx..=maxx {
                    let wa = edge(b, c, x as f64 + 0.5, y as f64 + 0.5) / area;
                    let wb = edge(c, a, x as f64 + 0.5, y as f64 + 0.5) / area;
                    let wc = 1. - wa - wb;
                    if wa < -1e-8 || wb < -1e-8 || wc < -1e-8 {
                        continue;
                    }
                    let z = if camera.orthographic {
                        wa * a[2] + wb * b[2] + wc * c[2]
                    } else {
                        1. / (wa / a[2] + wb / b[2] + wc / c[2])
                    };
                    let i = y * w + x;
                    if z > depth[i] + 1e-6 {
                        continue;
                    }
                    if hole && ((x + y) % 4 < 2) {
                        continue;
                    }
                    depth[i] = z;
                    frame.rgba[i * 4..i * 4 + 3].copy_from_slice(&rgb);
                    frame.ids[i] = id_index;
                }
            }
        }
    }
    if let Some(m) = result {
        draw(
            camera,
            m,
            [98, 200, 169],
            None,
            false,
            &mut frame,
            &mut depth,
        )
    } else {
        for hole in [false, true] {
            for s in sources.iter().filter(|s| s.hole == hole) {
                let color = if hole {
                    [232, 178, 106]
                } else if selected.contains(&s.id) {
                    s.color.map(|v| (v as u16 + 35).min(255) as u8)
                } else {
                    s.color
                };
                draw(
                    camera,
                    &s.mesh,
                    color,
                    (!s.id.is_empty()).then_some(s.id.as_str()),
                    hole,
                    &mut frame,
                    &mut depth,
                );
            }
        }
    }
    frame
}
