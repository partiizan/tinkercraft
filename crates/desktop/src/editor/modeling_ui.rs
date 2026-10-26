use super::*;
use tinkercraft_core::modeling::{self, Repeat};

pub(super) struct Tools {
    pub open: bool,
    pub group: Option<(Node, Node)>,
    pub ruler: [[f64; 3]; 2],
    pub show_ruler: bool,
    pub picking: u8,
    pub scene: bool,
    pub query: String,
    pub rename: String,
    pub reference: String,
    pub ref_axis: usize,
    pub source_mode: i32,
    pub target_mode: i32,
    pub offset: f64,
    pub translation: [f64; 3],
    pub rotation: [f64; 3],
    pub scale: f64,
    pub custom_pivot: bool,
    pub transform_pivot: [f64; 3],
    axis: usize,
    gaps: bool,
    copies: usize,
    radial: bool,
    step: [f64; 3],
    pivot: [f64; 3],
    sweep: f64,
    rotate: bool,
    segments: u32,
}
impl Default for Tools {
    fn default() -> Self {
        Self {
            open: false,
            group: None,
            ruler: [[0.; 3], [20., 0., 0.]],
            show_ruler: false,
            picking: 0,
            scene: false,
            query: String::new(),
            rename: String::new(),
            reference: String::new(),
            ref_axis: 0,
            source_mode: 0,
            target_mode: 0,
            offset: 0.,
            translation: [0.; 3],
            rotation: [0.; 3],
            scale: 1.,
            custom_pivot: false,
            transform_pivot: [0.; 3],
            axis: 0,
            gaps: false,
            copies: 3,
            radial: false,
            step: [25., 0., 0.],
            pivot: [0.; 3],
            sweep: 360.,
            rotate: true,
            segments: 96,
        }
    }
}
fn xyz(ui: &mut egui::Ui, values: &mut [f64; 3], range: std::ops::RangeInclusive<f64>) {
    ui.horizontal(|ui| {
        for (axis, v) in values.iter_mut().enumerate() {
            ui.label(["X", "Y", "Z"][axis]);
            ui.add(
                egui::DragValue::new(v)
                    .speed(0.1)
                    .range(range.clone())
                    .max_decimals(3),
            );
        }
    });
}
fn curved(n: &Node) -> bool {
    matches!(
        n.shape,
        Shape::Cylinder { .. }
            | Shape::Cone { .. }
            | Shape::Sphere { .. }
            | Shape::Tube { .. }
            | Shape::Torus { .. }
    )
}
fn resolution(ui: &mut egui::Ui, segments: &mut u32) {
    ui.horizontal(|ui| {
        ui.label("Curve resolution");
        for v in [24, 48, 96, 192] {
            ui.selectable_value(segments, v, v.to_string());
        }
    });
}
fn child_editor(ui: &mut egui::Ui, n: &mut Node, depth: usize) {
    egui::CollapsingHeader::new(format!(
        "{} · {}{}",
        n.name,
        n.kind(),
        if n.hole { " · Hole" } else { "" }
    ))
    .id_salt(&n.id)
    .default_open(depth == 0)
    .show(ui, |ui| {
        ui.add_enabled_ui(!n.hidden && !n.has_locked(), |ui| {
            ui.text_edit_singleline(&mut n.name);
            ui.checkbox(&mut n.hole, "Hole (subtracts within this group)");
            ui.label("Position relative to containing group · mm");
            xyz(ui, &mut n.position, -10000. ..=10000.);
            ui.label("Local rotation · degrees");
            xyz(ui, &mut n.rotation, -3600. ..=3600.);
            if let Some(size) = n.size() {
                ui.horizontal(|ui| {
                    ui.label("Dimensions · mm");
                    for (axis, old) in size.iter().enumerate() {
                        if (n.kind() == "Sphere" && axis > 0)
                            || (matches!(
                                n.shape,
                                Shape::Cylinder { .. }
                                    | Shape::Cone { .. }
                                    | Shape::Tube { .. }
                                    | Shape::Torus { .. }
                                    | Shape::Polygon { .. }
                            ) && axis == 1)
                        {
                            continue;
                        }
                        let mut value = *old;
                        ui.label(["X / diameter", "Y", "Z"][axis]);
                        if ui
                            .add(
                                egui::DragValue::new(&mut value)
                                    .speed(0.1)
                                    .range(0.01..=1000.),
                            )
                            .changed()
                        {
                            n.set_dimension(axis, value);
                        }
                    }
                });
            }
            super::advanced_ui::parameters(ui, n);
            if curved(n) {
                resolution(ui, &mut n.segments);
            }
            if !n.hole {
                let mut color = n.color;
                ui.horizontal(|ui| {
                    ui.label("Color");
                    if ui.color_edit_button_srgb(&mut color).changed() {
                        n.set_color_recursive(color);
                    }
                });
            }
            if let Shape::Group { scale, children } = &mut n.shape {
                ui.horizontal(|ui| {
                    ui.label("Uniform scale");
                    ui.add(
                        egui::DragValue::new(scale)
                            .speed(0.01)
                            .range(0.00001..=100000.),
                    );
                });
                for child in children {
                    child_editor(ui, child, depth + 1);
                }
            }
        });
        if n.hidden || n.has_locked() {
            ui.small("Show and unlock this group/component in the main editor before editing.");
        }
    });
}
impl Editor {
    pub(super) fn start_group_edit(&mut self) {
        self.commit_draft();
        if self.selection.len() != 1 {
            self.status = "Select one group to edit its contents".into();
            return;
        }
        if let Some(n) = self
            .history
            .project
            .parts
            .iter()
            .find(|n| self.selection.contains(&n.id))
        {
            if matches!(n.shape, Shape::Group { .. }) && !n.hidden && !n.has_locked() {
                self.tools.group = Some((n.clone(), n.clone()));
            } else {
                self.status = "Select a visible, unlocked group".into();
            }
        }
    }
    pub(super) fn modeling_tools(&mut self, ctx: &egui::Context) {
        if self.tools.open {
            let mut open = true;
            egui::Window::new("Modeling tools").open(&mut open).default_pos([230.,120.]).default_width(460.).vscroll(true).show(ctx,|ui|{
                self.advanced_tools(ui);
                ui.separator();
                ui.heading("Measurements · mm");
                ui.small("World-axis source bounds, before cuts; ruler points are independent of geometry.");
                if let Some(b)=modeling::selection_bounds(&self.history.project,&self.selection){
                    let s=b.max-b.min;ui.label(format!("Selection size: X {:.3}   Y {:.3}   Z {:.3}",s.x,s.y,s.z));
                    ui.label(format!("Minimum: {:.3}, {:.3}, {:.3}",b.min.x,b.min.y,b.min.z));
                    ui.label(format!("Maximum: {:.3}, {:.3}, {:.3}",b.max.x,b.max.y,b.max.z));
                }else{ui.label("Select objects to measure their bounds.");}
                let nodes:Vec<_>=self.history.project.parts.iter().filter(|n|self.selection.contains(&n.id)).collect();
                if nodes.len()==2 {
                    let a=geometry::bounds(nodes[0]);let b=geometry::bounds(nodes[1]);
                    if a.min.is_finite()&&a.max.is_finite()&&b.min.is_finite()&&b.max.is_finite(){
                        let gaps=modeling::bounds_gap(&a,&b);
                        ui.label(format!("Bounds-center distance: {:.3}",a.center().distance(b.center())));
                        ui.label(format!("Bounds gaps X/Y/Z: {:.3} / {:.3} / {:.3}",gaps.x,gaps.y,gaps.z));
                        ui.small("Bounding-box gaps are not exact curved/rotated surface clearance.");
                    }
                }
                ui.separator();ui.label("Ruler A · world coordinates");xyz(ui,&mut self.tools.ruler[0],-10000. ..=10000.);
                ui.label("Ruler B · world coordinates");xyz(ui,&mut self.tools.ruler[1],-10000. ..=10000.);
                let delta=DVec3::from(self.tools.ruler[1])-DVec3::from(self.tools.ruler[0]);
                ui.label(format!("Distance {:.3} mm · ΔX {:.3}  ΔY {:.3}  ΔZ {:.3}",delta.length(),delta.x,delta.y,delta.z));
                ui.checkbox(&mut self.tools.show_ruler,"Show ruler in viewport");
                ui.horizontal(|ui|{if ui.button("Pick two points on workplane").clicked(){self.tools.picking=1;self.tools.show_ruler=true;}
                    if ui.button("Clear ruler").clicked(){self.tools.picking=0;self.tools.show_ruler=false;}});
                if self.tools.picking>0{ui.label(format!("Click point {} on the active workplane; movement snap applies. Use Top view if edge-on.",self.tools.picking));}
                ui.separator();ui.heading("Distribute selection");
                ui.horizontal(|ui|{for (axis,label) in ["X","Y","Z"].iter().enumerate(){ui.selectable_value(&mut self.tools.axis,axis,*label);}});
                ui.checkbox(&mut self.tools.gaps,"Equal bounding-box gaps (otherwise equal centers)");
                if ui.add_enabled(self.selection.len()>=3,egui::Button::new("Distribute 3+ objects")).clicked(){
                    self.commit_draft();let axis=self.tools.axis;let gaps=self.tools.gaps;
                    self.operation(|p,s|{modeling::distribute(p,s,axis,gaps)?;Ok(None)});
                }
                ui.small("End objects stay fixed. Gaps can be negative when objects overlap.");
                ui.separator();ui.heading("Repeated copies");
                ui.horizontal(|ui|{ui.label("Additional copies");ui.add(egui::DragValue::new(&mut self.tools.copies).range(1..=99));ui.checkbox(&mut self.tools.radial,"Radial around world Z");});
                if self.tools.radial{ui.label("Pivot · world coordinates");xyz(ui,&mut self.tools.pivot,-10000. ..=10000.);
                    ui.horizontal(|ui|{ui.label("Sweep degrees");ui.add(egui::DragValue::new(&mut self.tools.sweep).range(-360. ..=360.));ui.checkbox(&mut self.tools.rotate,"Rotate copies");});
                    ui.small("360° closes the circle without a duplicate endpoint; partial arcs include the end angle.");
                }else{ui.label("Offset per copy · world millimeters");xyz(ui,&mut self.tools.step,-10000. ..=10000.);}
                if ui.add_enabled(!self.selection.is_empty(),egui::Button::new("Create copies")).clicked(){
                    self.commit_draft();let copies=self.tools.copies;
                    let pattern=if self.tools.radial{Repeat::Radial{pivot:self.tools.pivot,degrees:self.tools.sweep,rotate:self.tools.rotate}}else{Repeat::Linear(self.tools.step)};
                    self.operation(|p,s|Ok(Some(modeling::repeat(p,s,copies,pattern)?)));
                }
                ui.small("Selection is repeated as a set. Groups keep their cuts; copies receive new IDs. Maximum 100 total nodes.");
                ui.separator();ui.heading("Curved primitives");resolution(ui,&mut self.tools.segments);
                ui.small("Cylinder/cone/tube/torus segments; spheres use half as many longitudes. 96 preserves existing geometry. Higher values cost more processing.");
                if ui.button("Apply resolution to selected curved shapes").clicked(){
                    self.commit_draft();let segments=self.tools.segments;
                    self.operation(|p,s|{let mut count=0;for n in p.parts.iter_mut().filter(|n|s.contains(&n.id)&&curved(n)){n.segments=segments;count+=1;}
                        anyhow::ensure!(count>0,"Select a cylinder, cone, sphere, tube or torus; use Edit group contents for children");Ok(None)});
                }
            });
            self.tools.open = open;
        }
        if let Some((original, mut edited)) = self.tools.group.take() {
            let mut apply = false;
            let mut cancel = false;
            egui::Modal::new(egui::Id::new("group_contents")).show(ctx,|ui|{
                ui.set_width(650.);ui.heading(format!("Edit contents · {}",original.name));
                ui.label("Values are local to each containing group. Apply updates the Boolean result in one undo step.");
                ui.small(&self.status);
                egui::ScrollArea::vertical().max_height(480.).show(ui,|ui|{if let Shape::Group{children,..}=&mut edited.shape{for n in children{child_editor(ui,n,0);}}});
                ui.horizontal(|ui|{apply=ui.button("Apply changes").clicked();cancel=ui.button("Cancel").clicked();});
                if ctx.input(|i|i.key_pressed(egui::Key::Escape)){cancel=true;}
            });
            if apply {
                let mut p = self.history.project.clone();
                match modeling::edit_group(&mut p, &original, edited.clone()) {
                    Ok(()) => self.commit(p),
                    Err(e) => {
                        self.status = e.to_string();
                        self.tools.group = Some((original, edited));
                    }
                }
            } else if !cancel {
                self.tools.group = Some((original, edited));
            }
        }
    }
}
