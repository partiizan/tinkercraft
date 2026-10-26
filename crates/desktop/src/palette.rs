//! Compact, illustrated primitive buttons: labels remain readable alongside shape silhouettes.
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

pub fn button(ui: &mut egui::Ui, kind: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(90., 44.), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    let painter = ui.painter();
    painter.rect(
        rect,
        4.,
        visuals.bg_fill,
        visuals.bg_stroke,
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center_top() + Vec2::new(0., 3.),
        egui::Align2::CENTER_TOP,
        kind,
        egui::FontId::proportional(13.),
        visuals.text_color(),
    );
    let center = rect.center_top() + Vec2::new(0., 30.);
    let light = Color32::from_rgb(131, 185, 183);
    let mid = Color32::from_rgb(89, 148, 148);
    let dark = Color32::from_rgb(62, 105, 114);
    let point = |x, y| center + Vec2::new(x, y);
    let polygon = |points: &[(f32, f32)], color| {
        painter.add(egui::Shape::convex_polygon(
            points.iter().map(|&(x, y)| point(x, y)).collect(),
            color,
            Stroke::NONE,
        ));
    };
    let ellipse = |y: f32, color| {
        let points: Vec<Pos2> = (0..32)
            .map(|i| {
                let a = i as f32 * std::f32::consts::TAU / 32.;
                point(11. * a.cos(), y + 3.5 * a.sin())
            })
            .collect();
        painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
    };
    match kind {
        "Tube" | "Torus" => {
            painter.circle_filled(center, 10., mid);
            painter.circle_filled(center, 5., visuals.bg_fill);
            painter.circle_stroke(center, 9., Stroke::new(2_f32, light));
        }
        "Polygon" => {
            polygon(
                &[
                    (-11., 0.),
                    (-5., -9.),
                    (5., -9.),
                    (11., 0.),
                    (5., 9.),
                    (-5., 9.),
                ],
                light,
            );
        }
        "Text" => {
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "Ab",
                egui::FontId::monospace(20.),
                light,
            );
        }
        "Sphere" => {
            painter.circle_filled(center, 10., dark);
            painter.circle_filled(center + Vec2::new(-1.5, -1.5), 8., mid);
            painter.circle_filled(center + Vec2::new(-3., -3.), 4., light);
        }
        "Cylinder" => {
            ellipse(7., dark);
            polygon(&[(-11., -7.), (11., -7.), (11., 7.), (-11., 7.)], mid);
            ellipse(-7., light);
        }
        "Cone" => {
            ellipse(7., dark);
            polygon(&[(0., -11.), (11., 7.), (-11., 7.)], mid);
            polygon(&[(0., -11.), (0., 8.), (-11., 7.)], light);
        }
        "Wedge" => {
            polygon(&[(-12., -8.), (-12., 7.), (4., 11.), (4., -4.)], dark);
            polygon(&[(-12., -8.), (4., -4.), (13., 6.), (-3., 2.)], light);
            polygon(&[(4., -4.), (13., 6.), (4., 11.)], mid);
        }
        "Roof" => {
            polygon(&[(-13., 5.), (-5., -10.), (12., -5.), (4., 10.)], light);
            polygon(&[(12., -5.), (15., 7.), (4., 10.)], dark);
        }
        _ => {
            polygon(&[(-11., -5.), (0., -10.), (11., -5.), (0., 0.)], light);
            polygon(&[(-11., -5.), (0., 0.), (0., 11.), (-11., 6.)], mid);
            polygon(&[(0., 0.), (11., -5.), (11., 6.), (0., 11.)], dark);
        }
    }
    response.on_hover_text(format!("Add {kind}"))
}
