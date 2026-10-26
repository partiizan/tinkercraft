//! Bounded SVG and font outline extrusion. Generated geometry is embedded in the project.
use crate::{Node, geometry, stl};
use anyhow::{Context, Result, ensure};
use clipper2_rust::{FillRule, PathD, PathsD, Point, union_subjects_d};
use manifold_rust::{linalg::Vec2, manifold::Manifold};
use std::{io::Read, path::Path as FilePath};
use usvg::tiny_skia_path::{Path, PathSegment};
const MAX_POINTS: usize = 12000;
#[derive(Default)]
struct Outline {
    contours: Vec<Vec<[f64; 2]>>,
    current: Vec<[f64; 2]>,
    last: [f64; 2],
    points: usize,
    overflow: bool,
}
impl Outline {
    fn push(&mut self, p: [f64; 2]) {
        self.points += 1;
        if self.points > MAX_POINTS {
            self.overflow = true;
            return;
        }
        if self.current.last() != Some(&p) {
            self.current.push(p);
        }
        self.last = p;
    }
    fn close(&mut self) {
        if self.current.first() == self.current.last() {
            self.current.pop();
        }
        if self.current.len() >= 3 {
            self.contours.push(std::mem::take(&mut self.current));
        } else {
            self.current.clear();
        }
    }
    fn start(&mut self, p: [f64; 2]) {
        self.close();
        self.push(p);
    }
    fn quad(&mut self, c: [f64; 2], p: [f64; 2]) {
        let a = self.last;
        for i in 1..=16 {
            let t = i as f64 / 16.;
            let s = 1. - t;
            self.push([
                s * s * a[0] + 2. * s * t * c[0] + t * t * p[0],
                s * s * a[1] + 2. * s * t * c[1] + t * t * p[1],
            ]);
        }
    }
    fn cubic(&mut self, c: [f64; 2], d: [f64; 2], p: [f64; 2]) {
        let a = self.last;
        for i in 1..=24 {
            let t = i as f64 / 24.;
            let s = 1. - t;
            self.push([
                s * s * s * a[0] + 3. * s * s * t * c[0] + 3. * s * t * t * d[0] + t * t * t * p[0],
                s * s * s * a[1] + 3. * s * s * t * c[1] + 3. * s * t * t * d[1] + t * t * t * p[1],
            ]);
        }
    }
    fn from_path(p: &Path) -> Result<Self> {
        let mut out = Self::default();
        for s in p.segments() {
            match s {
                PathSegment::MoveTo(p) => out.start([p.x as f64, p.y as f64]),
                PathSegment::LineTo(p) => out.push([p.x as f64, p.y as f64]),
                PathSegment::QuadTo(c, p) => {
                    out.quad([c.x as f64, c.y as f64], [p.x as f64, p.y as f64])
                }
                PathSegment::CubicTo(c, d, p) => out.cubic(
                    [c.x as f64, c.y as f64],
                    [d.x as f64, d.y as f64],
                    [p.x as f64, p.y as f64],
                ),
                PathSegment::Close => out.close(),
            }
        }
        out.close();
        ensure!(
            !out.overflow,
            "Outline exceeds 12,000 points; simplify the artwork"
        );
        Ok(out)
    }
}
impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start([x as f64, y as f64]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.push([x as f64, y as f64]);
    }
    fn quad_to(&mut self, x: f32, y: f32, xx: f32, yy: f32) {
        self.quad([x as f64, y as f64], [xx as f64, yy as f64]);
    }
    fn curve_to(&mut self, x: f32, y: f32, xx: f32, yy: f32, xxx: f32, yyy: f32) {
        self.cubic(
            [x as f64, y as f64],
            [xx as f64, yy as f64],
            [xxx as f64, yyy as f64],
        );
    }
    fn close(&mut self) {
        Outline::close(self);
    }
}
pub fn read_bounded(path: &FilePath, limit: usize) -> Result<Vec<u8>> {
    let mut b = vec![];
    std::fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut b)?;
    ensure!(b.len() <= limit, "File exceeds {} MB", limit / 1_000_000);
    Ok(b)
}
fn paths(contours: Vec<Vec<[f64; 2]>>) -> PathsD {
    contours
        .into_iter()
        .map(|c| {
            c.into_iter()
                .map(|p| Point::new(p[0], p[1]))
                .collect::<PathD>()
        })
        .collect()
}
fn extrude(contours: PathsD, width: f64, height: f64, name: &str) -> Result<Node> {
    ensure!(
        width.is_finite()
            && (0.1..=1000.).contains(&width)
            && height.is_finite()
            && (0.01..=1000.).contains(&height),
        "Width must be 0.1–1000 mm and extrusion 0.01–1000 mm"
    );
    ensure!(!contours.is_empty(), "No filled outline to extrude");
    ensure!(
        contours.iter().map(Vec::len).sum::<usize>() <= MAX_POINTS,
        "Combined outline exceeds 12,000 points"
    );
    let (min, max) = contours.iter().flatten().fold(
        ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]),
        |(mut a, mut b), p| {
            a[0] = a[0].min(p.x);
            a[1] = a[1].min(p.y);
            b[0] = b[0].max(p.x);
            b[1] = b[1].max(p.y);
            (a, b)
        },
    );
    let scale = width / (max[0] - min[0]);
    ensure!(scale.is_finite() && max[1] > min[1], "Empty outline bounds");
    let polygons = contours
        .iter()
        .map(|c| {
            c.iter()
                .map(|p| Vec2::new((p.x - min[0]) * scale, (p.y - min[1]) * scale))
                .collect()
        })
        .collect();
    let solid = Manifold::extrude(&polygons, height, 0, 0., Vec2::new(1., 1.));
    let mesh = geometry::mesh(&solid);
    ensure!(
        mesh.triangles.len() <= stl::MAX_TRIANGLES,
        "Extrusion exceeds 20,000 triangles; simplify artwork or text"
    );
    let mut node = stl::import(&mesh.stl()?, name, 1.)?;
    node.position = [40., 15., height / 2.];
    Ok(node)
}
/// Filled paths and expanded strokes, with SVG transforms and winding rules.
/// Effects/raster/text are explicitly rejected instead of silently changing their shape.
pub fn svg(bytes: &[u8], width: f64, height: f64, name: &str) -> Result<Node> {
    ensure!(bytes.len() <= 1_000_000, "SVG exceeds 1 MB");
    let source = std::str::from_utf8(bytes)?;
    let xml = roxmltree::Document::parse(source)?;
    let mut count = 0;
    for n in xml.descendants().filter(|n| n.is_element()) {
        count += 1;
        ensure!(
            count <= 2000 && n.ancestors().count() <= 32,
            "SVG structure exceeds limits"
        );
        ensure!(
            !matches!(
                n.tag_name().name(),
                "image" | "text" | "foreignObject" | "script" | "filter" | "mask" | "clipPath"
            ),
            "Convert text/effects/clips to plain vector paths before importing"
        );
        for a in n.attributes() {
            ensure!(
                a.name() != "href" || a.value().starts_with('#'),
                "External SVG references are unsupported"
            );
        }
    }
    let mut options = usvg::Options::default();
    options.image_href_resolver.resolve_data = Box::new(|_, _, _| None);
    options.image_href_resolver.resolve_string = Box::new(|_, _| None);
    let tree = usvg::Tree::from_data(bytes, &options).context("Invalid SVG")?;
    fn collect(g: &usvg::Group, out: &mut PathsD, points: &mut usize) -> Result<()> {
        if g.opacity().get() == 0. {
            return Ok(());
        }
        for n in g.children() {
            match n {
                usvg::Node::Group(g) => collect(g, out, points)?,
                usvg::Node::Path(p) => {
                    if !p.is_visible() {
                        continue;
                    }
                    let mut add = |path: Path, rule: FillRule| -> Result<()> {
                        let path = path
                            .transform(p.abs_transform())
                            .ok_or_else(|| anyhow::anyhow!("Invalid SVG transform"))?;
                        let mut outline = Outline::from_path(&path)?;
                        *points += outline.points;
                        ensure!(*points <= MAX_POINTS, "SVG exceeds 12,000 outline points");
                        for c in &mut outline.contours {
                            for v in c {
                                v[1] = -v[1];
                            }
                        }
                        out.extend(union_subjects_d(&paths(outline.contours), rule, 6));
                        Ok(())
                    };
                    if let Some(fill) = p.fill()
                        && fill.opacity().get() > 0.
                    {
                        add(
                            p.data().clone(),
                            if fill.rule() == usvg::FillRule::EvenOdd {
                                FillRule::EvenOdd
                            } else {
                                FillRule::NonZero
                            },
                        )?;
                    }
                    if let Some(stroke) = p.stroke()
                        && stroke.opacity().get() > 0.
                    {
                        let path = p
                            .data()
                            .stroke(&stroke.to_tiny_skia(), 1.)
                            .ok_or_else(|| anyhow::anyhow!("Unable to expand SVG stroke"))?;
                        add(path, FillRule::NonZero)?;
                    }
                }
                _ => anyhow::bail!("Only vector paths are supported"),
            }
        }
        Ok(())
    }
    let mut result = vec![];
    collect(tree.root(), &mut result, &mut 0)?;
    extrude(
        union_subjects_d(&result, FillRule::NonZero, 6),
        width,
        height,
        name,
    )
}
/// Outline Latin text using a chosen TTF/OTF face. Output embeds geometry, not font files.
pub fn font(bytes: &[u8], text: &str, width: f64, height: f64) -> Result<Node> {
    ensure!(bytes.len() <= 20_000_000, "Font exceeds 20 MB");
    ensure!(
        !text.trim().is_empty() && text.chars().count() <= 48,
        "Use 1–48 characters"
    );
    ensure!(
        text.chars().all(|c| (' '..='\u{024f}').contains(&c)),
        "Font lettering supports Latin text; complex scripts require SVG outlines"
    );
    let face = ttf_parser::Face::parse(bytes, 0)
        .map_err(|_| anyhow::anyhow!("Unsupported TTF/OTF font"))?;
    let mut result = vec![];
    let mut offset = 0.;
    let mut previous = None;
    let mut total = 0;
    for c in text.chars() {
        let glyph = face
            .glyph_index(c)
            .ok_or_else(|| anyhow::anyhow!("Font does not contain {c}"))?;
        if let (Some(prev), Some(kern)) = (previous, face.tables().kern) {
            for table in kern.subtables {
                if table.horizontal && !table.variable {
                    offset += table.glyphs_kerning(prev, glyph).unwrap_or(0) as f64;
                }
            }
        }
        let mut o = Outline::default();
        face.outline_glyph(glyph, &mut o);
        o.close();
        ensure!(!o.overflow, "Glyph exceeds outline limit");
        total += o.points;
        ensure!(
            total <= MAX_POINTS,
            "Text exceeds 12,000 outline points; use fewer characters"
        );
        for contour in &mut o.contours {
            for v in contour {
                v[0] += offset;
            }
        }
        result.extend(paths(o.contours));
        offset += face.glyph_hor_advance(glyph).unwrap_or(face.units_per_em()) as f64;
        previous = Some(glyph);
    }
    extrude(
        union_subjects_d(&result, FillRule::NonZero, 6),
        width,
        height,
        "Font lettering",
    )
}
