use super::*;
use tinkercraft_core::{extrusion, spatial};
pub(super) struct Scope {
    pub parent: History,
    pub id: String,
    pub camera: Camera,
    pub selection: Selection,
}
pub(super) struct Studio {
    pub open: bool,
    pub help: bool,
    pub gpu_enabled: bool,
    pub plane: spatial::Workplane,
    pub pick_mode: u8,
    pub anchor: Option<(DVec3, u64)>,
    pub vertex_snap: bool,
    pub marquee: Option<Pos2>,
    pub scopes: Vec<Scope>,
    pub svg_path: Option<PathBuf>,
    pub font_path: Option<PathBuf>,
    pub text: String,
    pub width: f64,
    pub depth: f64,
    pub job: Option<mpsc::Receiver<Result<Node, String>>>,
    pub pending_node: Option<Node>,
    pub error: String,
    pub zoom: f32,
}
impl Default for Studio {
    fn default() -> Self {
        Self {
            open: false,
            help: false,
            gpu_enabled: true,
            plane: spatial::Workplane::default(),
            pick_mode: 0,
            anchor: None,
            vertex_snap: false,
            marquee: None,
            scopes: vec![],
            svg_path: None,
            font_path: None,
            text: "TinkerCraft".into(),
            width: 60.,
            depth: 2.,
            job: None,
            pending_node: None,
            error: String::new(),
            zoom: 1.,
        }
    }
}
fn xyz(ui: &mut egui::Ui, label: &str, v: &mut [f64; 3]) {
    ui.label(label);
    ui.horizontal(|ui| {
        for (i, x) in v.iter_mut().enumerate() {
            ui.label(["X", "Y", "Z"][i]);
            ui.add(egui::DragValue::new(x).speed(0.1).range(-10000. ..=10000.));
        }
    });
}
impl Editor {
    pub(super) fn full_project(&self, p: &Project) -> anyhow::Result<Project> {
        let mut p = p.clone();
        for scope in self.studio.scopes.iter().rev() {
            p = spatial::replace_children(&scope.parent.project, &scope.id, p.parts)?;
        }
        Ok(p)
    }
    pub(super) fn enter_group(&mut self) {
        self.commit_draft();
        if self.selection.len() != 1 {
            return;
        }
        let Some(n) = self
            .history
            .project
            .parts
            .iter()
            .find(|n| self.selection.contains(&n.id))
            .cloned()
        else {
            return;
        };
        let Shape::Group { children, .. } = n.shape else {
            self.status = "Select a group to enter".into();
            return;
        };
        if n.hidden || n.locked || children.iter().any(Node::has_locked) {
            self.status = "Show and unlock the group first".into();
            return;
        }
        let p = Project {
            parts: children,
            name: self.history.project.name.clone(),
            ..Project::default()
        };
        let local = History::new(p, !self.history.dirty());
        let parent = std::mem::replace(&mut self.history, local);
        self.studio.scopes.push(Scope {
            parent,
            id: n.id,
            camera: self.camera.clone(),
            selection: self.selection.clone(),
        });
        self.selection.clear();
        self.studio.pick_mode = 0;
        self.refresh();
        self.fit(1.4);
    }
    pub(super) fn leave_group(&mut self, apply: bool) -> bool {
        self.commit_draft();
        let Some(scope) = self.studio.scopes.last() else {
            return true;
        };
        let p = if apply {
            match spatial::replace_children(
                &scope.parent.project,
                &scope.id,
                self.history.project.parts.clone(),
            ) {
                Ok(p) => Some(p),
                Err(e) => {
                    self.status = e.to_string();
                    return false;
                }
            }
        } else {
            None
        };
        let scope = self.studio.scopes.pop().unwrap();
        self.history = scope.parent;
        self.camera = scope.camera;
        self.selection = scope.selection;
        if let Some(p) = p
            && let Err(e) = self.history.commit(p)
        {
            self.status = e.to_string();
            return false;
        }
        self.studio.pick_mode = 0;
        self.refresh();
        true
    }
    pub(super) fn finish_scopes(&mut self) -> bool {
        while !self.studio.scopes.is_empty() {
            if !self.leave_group(true) {
                return false;
            }
        }
        true
    }
    pub(super) fn place_on_plane(&self, n: &mut Node) {
        let plane = &self.history.project.workplane;
        if *plane != spatial::Workplane::default() {
            n.rotation = plane.rotation;
            n.position = plane
                .world(DVec3::new(
                    0.,
                    0.,
                    n.size().map(|s| s[2] / 2.).unwrap_or(0.),
                ))
                .to_array();
        }
    }
    pub(super) fn spatial_click(&mut self, q: DVec2, width: usize, height: usize, shift: bool) {
        let (o, d) = self.camera.ray(q, width as f64, height as f64);
        if self.tools.picking > 0 {
            if let Some(point) = self
                .history
                .project
                .workplane
                .intersect(o, d, self.move_snap)
            {
                self.tools.ruler[(self.tools.picking - 1) as usize] = point.to_array();
                self.tools.picking = if self.tools.picking == 1 { 2 } else { 0 };
            } else {
                self.status = "Active workplane is edge-on or behind the camera".into();
            }
            return;
        }
        let sources: Vec<_> = self
            .sources
            .iter()
            .filter(|s| match self.studio.pick_mode {
                1 => !s.hole,
                2 => self.selection.contains(&s.id),
                3 => !self.selection.contains(&s.id),
                _ => true,
            })
            .cloned()
            .collect();
        let Some(hit) = spatial::pick(&sources, o, d) else {
            return;
        };
        match self.studio.pick_mode {
            1 => {
                match spatial::Workplane::on_face(hit.point, hit.normal) {
                    Ok(plane) => {
                        let mut p = self.history.project.clone();
                        p.workplane = plane;
                        self.commit(p);
                    }
                    Err(e) => self.status = e.to_string(),
                }
                self.studio.pick_mode = 0;
            }
            2 => {
                self.studio.anchor = Some((
                    spatial::snap_point(&hit, self.studio.vertex_snap),
                    self.revision,
                ));
                self.studio.pick_mode = 3;
                self.status = "Click a target on another object".into();
            }
            3 => {
                if let Some((source, revision)) = self.studio.anchor.take() {
                    if revision == self.revision {
                        let delta = (spatial::snap_point(&hit, self.studio.vertex_snap) - source)
                            .to_array();
                        self.operation(|p, s| {
                            operations::translate(p, s, delta)?;
                            Ok(None)
                        });
                    } else {
                        self.status = "Scene changed; start snapping again".into();
                    }
                }
                self.studio.pick_mode = 0;
            }
            _ => self.select(hit.id, shift),
        }
    }
    fn generate_outline(&mut self, font: bool, ctx: &egui::Context) {
        self.commit_draft();
        let path = if font {
            self.studio.font_path.clone()
        } else {
            self.studio.svg_path.clone()
        };
        let Some(path) = path else {
            self.studio.error = "Choose a file first".into();
            return;
        };
        let (w, h, text) = (
            self.studio.width,
            self.studio.depth,
            self.studio.text.clone(),
        );
        let (tx, rx) = mpsc::channel();
        self.studio.job = Some(rx);
        self.studio.error.clear();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| {
                let bytes =
                    extrusion::read_bounded(&path, if font { 20_000_000 } else { 1_000_000 })?;
                if font {
                    extrusion::font(&bytes, &text, w, h)
                } else {
                    extrusion::svg(
                        &bytes,
                        w,
                        h,
                        path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("SVG outline"),
                    )
                }
            })
            .map_err(|_| "Outline conversion failed".to_string())
            .and_then(|r: anyhow::Result<Node>| r.map_err(|e| e.to_string()));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    pub(super) fn studio_ui(&mut self, ctx: &egui::Context) {
        if let Some(job) = &self.studio.job {
            if let Ok(result) = job.try_recv() {
                self.studio.job = None;
                match result {
                    Ok(n) => self.studio.pending_node = Some(n),
                    Err(e) => self.studio.error = e,
                }
            } else {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
        if !self.studio.scopes.is_empty() {
            egui::Window::new("Editing group contents in local coordinates")
                .anchor(egui::Align2::CENTER_TOP, [0., 90.])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Depth {} · solids and holes are editable here",
                        self.studio.scopes.len()
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Apply and exit (Shift-F3)").clicked() {
                            self.leave_group(true);
                        }
                        if ui.button("Cancel group edits").clicked() {
                            self.leave_group(false);
                        }
                    });
                });
        }
        if self.studio.open {
            let mut open = true;
            egui::Window::new("Modeling studio").open(&mut open).default_pos([240.,130.]).default_width(480.).vscroll(true).show(ctx,|ui|{
   ui.heading("Workplane");ui.small("Saved with the project. New shapes, drop and ruler picks use this plane; numeric object positions remain world coordinates.");
   if ui.button("Read active plane").clicked(){self.studio.plane=self.history.project.workplane.clone();}
   xyz(ui,"Origin · mm",&mut self.studio.plane.origin);xyz(ui,"Rotation · degrees",&mut self.studio.plane.rotation);
   ui.horizontal(|ui|{if ui.button("Apply plane").clicked(){self.commit_draft();let mut p=self.history.project.clone();p.workplane=self.studio.plane.clone();self.commit(p);}
if ui.button("Pick a face").clicked(){self.commit_draft();self.studio.pick_mode=1;self.status="Click a solid face to place the workplane".into();}
if ui.button("Reset XY").clicked(){let mut p=self.history.project.clone();p.workplane=spatial::Workplane::default();self.studio.plane=p.workplane.clone();self.commit(p);}});
   if ui.button("Look at workplane").clicked(){let plane=&self.history.project.workplane;let normal=plane.normal();self.camera.yaw=normal.y.atan2(normal.x);self.camera.pitch=normal.z.asin();self.camera.target=DVec3::from(plane.origin);self.redraw=true;}
   ui.separator();ui.heading("Snap objects");ui.checkbox(&mut self.studio.vertex_snap,"Snap to nearest triangle vertex (otherwise surface point)");if ui.button("Pick source, then target").clicked(){self.commit_draft();if self.selection.is_empty(){self.status="Select an object first".into();}else{self.studio.pick_mode=2;self.status="Click a source point on the selection, then another object's target point".into();}}
   ui.small("Moves the selection as one set. Escape cancels. Shift-drag in the viewport selects objects by their bounds centers; Alt-Shift-drag removes them.");
   ui.separator();ui.heading("Group viewport editing");ui.horizontal(|ui|{if ui.button("Enter selected group (F3)").clicked(){self.enter_group();}
if ui.button("Apply all group edits").clicked(){self.finish_scopes();}});ui.small("Nested groups can be entered again. Save/export applies all scopes first; crash recovery always stores the full project.");
   ui.separator();ui.heading("Outline extrusion");ui.horizontal(|ui|{ui.label("Width mm");ui.add(egui::DragValue::new(&mut self.studio.width).range(0.1..=1000.));ui.label("Depth mm");ui.add(egui::DragValue::new(&mut self.studio.depth).range(0.01..=1000.));});
   ui.add_enabled_ui(self.studio.job.is_none(),|ui|{
    ui.horizontal(|ui|{if ui.button("Choose SVG").clicked(){self.studio.svg_path=rfd::FileDialog::new().set_title("Choose SVG outline").add_filter("SVG",&["svg"]).pick_file();}
if ui.button("Generate SVG mesh").clicked(){self.generate_outline(false,ctx);}});if let Some(p)=&self.studio.svg_path{ui.small(p.file_name().unwrap_or_default().to_string_lossy());}
    ui.small("Paths/shapes, transforms, fills and strokes. Convert text, clipping, masks and effects to plain paths first. Colors become one editable object color.");
    ui.horizontal(|ui|{if ui.button("Choose TTF / OTF").clicked(){self.studio.font_path=rfd::FileDialog::new().set_title("Choose lettering font").add_filter("Outline font",&["ttf","otf"]).pick_file();}
if ui.button("Generate font mesh").clicked(){self.generate_outline(true,ctx);}});if let Some(p)=&self.studio.font_path{ui.small(p.file_name().unwrap_or_default().to_string_lossy());}ui.text_edit_singleline(&mut self.studio.text);ui.small("Latin lettering with font outlines and legacy kerning. Generated meshes are self-contained; regenerate to change the text/font.");
   });if self.studio.job.is_some(){ui.label("Generating mesh…");}
if !self.studio.error.is_empty(){ui.colored_label(Color32::LIGHT_RED,&self.studio.error);}
   if let Some(n)=&self.studio.pending_node{let size=n.size().unwrap();ui.label(format!("Ready: {:.2} × {:.2} × {:.2} mm",size[0],size[1],size[2]));ui.horizontal(|ui|{if ui.button("Add generated mesh").clicked(){self.commit_draft();if let Some(mut n)=self.studio.pending_node.take(){self.place_on_plane(&mut n);let id=n.id.clone();let mut p=self.history.project.clone();p.parts.push(n);self.selection=Selection::from([id]);self.commit(p);}}
if ui.button("Discard preview").clicked(){self.studio.pending_node=None;}});}
   ui.separator();ui.heading("Display and navigation");ui.small(if self.gpu.is_some(){"Native GPU available"}else{"GPU unavailable: software fallback active"});if ui.checkbox(&mut self.studio.gpu_enabled,"GPU viewport (disable for software fallback)").changed(){self.redraw=true;}
   if ui.add(egui::Slider::new(&mut self.studio.zoom,0.8..=1.6).text("Interface scale")).changed(){ctx.set_zoom_factor(self.studio.zoom);}
   if ui.button("Keyboard reference (F1)").clicked(){self.studio.help=true;}
  });
            self.studio.open = open;
        }
        if self.studio.help {
            let mut open = true;
            egui::Window::new("Keyboard reference").open(&mut open).show(ctx,|ui|{ui.label("Tab / Shift-Tab: focus controls; Space / Enter: activate\nCtrl/Cmd S: save · Shift-S: Save as · O: open\nCtrl/Cmd Z / Shift-Z: undo / redo\nCtrl/Cmd A: select all · D: duplicate · G: group · Shift-G: ungroup\nW / E / R: move / rotate / resize · arrows / Page Up/Down: nudge\nF: fit selection · 1/2/3/4: top/front/right/isometric\nM: modeling tools · F2: numeric group editor\nF3: enter group viewport · Shift-F3: apply and leave\nShift-drag: add box selection · Alt-Shift-drag: subtract\nEscape: cancel drag, snapping or ruler picking\nF1: this reference");});
            self.studio.help = open;
        }
    }
}
