use super::*;
use tinkercraft_core::modeling;

/// Parameter changes are committed by the inspector, or staged by the group modal.
pub(super) fn parameters(ui: &mut egui::Ui, n: &mut Node) -> bool {
    match &mut n.shape {
        Shape::Tube { bore, .. } | Shape::Torus { bore, .. } => {
            ui.label("Bore / outer diameter");
            ui.add(egui::Slider::new(bore, 0.05..=0.95)).changed()
        }
        Shape::Polygon { sides, .. } => {
            ui.label("Polygon sides");
            ui.add(egui::DragValue::new(sides).range(3..=32)).changed()
        }
        Shape::Text { content, .. } => {
            ui.label("Block lettering");
            let r = ui.text_edit_singleline(content);
            ui.small("A–Z, 0–9, spaces, - . _ + · up to 24 characters. Width fits the full label.");
            r.lost_focus()
        }
        _ => false,
    }
}
fn vector(ui: &mut egui::Ui, label: &str, v: &mut [f64; 3], range: std::ops::RangeInclusive<f64>) {
    ui.label(label);
    ui.horizontal(|ui| {
        for (axis, value) in v.iter_mut().enumerate() {
            ui.label(["X", "Y", "Z"][axis]);
            ui.add(egui::DragValue::new(value).range(range.clone()).speed(0.1));
        }
    });
}
fn edge(ui: &mut egui::Ui, id: &str, label: &str, value: &mut i32) {
    ui.horizontal(|ui| {
        ui.label(label);
        egui::ComboBox::from_id_salt(id)
            .selected_text(match *value {
                -1 => "Min",
                1 => "Max",
                _ => "Center",
            })
            .show_ui(ui, |ui| {
                for (v, s) in [(-1, "Min"), (0, "Center"), (1, "Max")] {
                    ui.selectable_value(value, v, s);
                }
            });
    });
}
fn tree(ui: &mut egui::Ui, n: &Node) {
    if let Shape::Group { children, .. } = &n.shape {
        egui::CollapsingHeader::new(format!("{} · {} components", n.name, children.len()))
            .id_salt(("tree", &n.id))
            .show(ui, |ui| {
                for c in children {
                    tree(ui, c);
                }
            });
    } else {
        ui.label(format!(
            "{} · {}{}",
            n.name,
            n.kind(),
            if n.hole { " · hole" } else { "" }
        ));
    }
}
impl Editor {
    pub(super) fn add_catalog_shape(&mut self, kind: &str) {
        self.commit_draft();
        let mut n = Node::primitive(kind);
        n.position[2] = n.size().unwrap()[2] / 2.;
        if kind == "Text" {
            let solids: Selection = self
                .history
                .project
                .parts
                .iter()
                .filter(|n| self.selection.contains(&n.id) && !n.hole && !n.hidden)
                .map(|n| n.id.clone())
                .collect();
            if let Some(b) = modeling::selection_bounds(&self.history.project, &solids) {
                n.position = [
                    b.center().x,
                    b.center().y,
                    b.max.z + n.size().unwrap()[2] / 2. - 0.2,
                ];
            }
        }
        let id = n.id.clone();
        let mut p = self.history.project.clone();
        p.parts.push(n);
        self.selection = Selection::from([id]);
        self.commit(p);
    }
    pub(super) fn advanced_tools(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("New shapes and lettering").show(ui,|ui|{
            ui.horizontal(|ui|{for kind in ["Tube","Torus","Polygon","Text"]{if crate::palette::button(ui,kind).clicked(){self.add_catalog_shape(kind);}}});
            ui.small("Set bore ratio, sides or label text in the inspector; F2 edits group components. Lettering can be Solid or Hole.");
        });
        egui::CollapsingHeader::new("Transform selection together").show(ui,|ui|{
            ui.small("Objects stay independent. Uniform scale and world rotation happen around the pivot, then movement is applied.");
            vector(ui,"Move · mm",&mut self.tools.translation,-10000. ..=10000.);
            vector(ui,"Rotate · degrees",&mut self.tools.rotation,-3600. ..=3600.);
            ui.horizontal(|ui|{ui.label("Uniform scale multiplier");ui.add(egui::DragValue::new(&mut self.tools.scale).range(0.00001..=100000.).speed(0.01));});
            ui.checkbox(&mut self.tools.custom_pivot,"Use custom world pivot");
            if self.tools.custom_pivot{vector(ui,"Pivot · mm",&mut self.tools.transform_pivot,-10000. ..=10000.);}else{ui.small("Pivot: center of combined source bounds.");}
            if ui.button("Apply selection transform").clicked(){self.commit_draft();let pivot=if self.tools.custom_pivot{Some(self.tools.transform_pivot)}else{modeling::selection_bounds(&self.history.project,&self.selection).map(|b|b.center().to_array())};
                if let Some(pivot)=pivot{let(delta,r,s)=(self.tools.translation,self.tools.rotation,self.tools.scale);self.operation(|p,ids|{modeling::transform_selection(p,ids,pivot,delta,r,s)?;Ok(None)});}
            }
            if ui.button("Reset transform inputs").clicked(){self.tools.translation=[0.;3];self.tools.rotation=[0.;3];self.tools.scale=1.;}
        });
        egui::CollapsingHeader::new("Align to reference object").show(ui,|ui|{
            ui.small("Reference stays fixed. Uses source bounding boxes before cuts; offset is along the chosen world axis.");
            let selected_name=self.history.project.parts.iter().find(|n|n.id==self.tools.reference).map(|n|n.name.as_str()).unwrap_or("Choose reference");
            egui::ComboBox::from_id_salt("reference_object").selected_text(selected_name).show_ui(ui,|ui|{for n in self.history.project.parts.iter().filter(|n|!n.hidden){ui.selectable_value(&mut self.tools.reference,n.id.clone(),format!("{} ({})",n.name,n.kind()));}});
            ui.horizontal(|ui|{for(axis,label)in["X","Y","Z"].iter().enumerate(){ui.selectable_value(&mut self.tools.ref_axis,axis,*label);}});
            edge(ui,"source_edge","Selected objects",&mut self.tools.source_mode);edge(ui,"target_edge","Reference",&mut self.tools.target_mode);
            ui.horizontal(|ui|{ui.label("Offset · mm");ui.add(egui::DragValue::new(&mut self.tools.offset).range(-10000. ..=10000.).speed(0.1));});
            if ui.button("Align to reference").clicked(){self.commit_draft();let mut p=self.history.project.clone();match modeling::align_reference(&mut p,&self.selection,&self.tools.reference,self.tools.ref_axis,self.tools.source_mode,self.tools.target_mode,self.tools.offset){Ok(())=>self.commit(p),Err(e)=>self.status=e.to_string()}}
        });
    }
    fn sync_browser_selection(&mut self) {
        self.draft = if self.selection.len() == 1 {
            self.history
                .project
                .parts
                .iter()
                .find(|n| self.selection.contains(&n.id))
                .cloned()
        } else {
            None
        };
        self.preview = false;
        self.redraw = true;
    }
    pub(super) fn scene_browser(&mut self, ctx: &egui::Context) {
        if !self.tools.scene {
            return;
        }
        let mut open = true;
        egui::Window::new("Object browser").open(&mut open).default_width(420.).vscroll(true).show(ctx,|ui|{
            ui.label("Search names, shape types or nested components");ui.text_edit_singleline(&mut self.tools.query);
            let matches:Vec<_>=self.history.project.parts.iter().filter(|n|modeling::matches_query(n,&self.tools.query)).cloned().collect();
            ui.label(format!("{} matches · {} selected",matches.len(),self.selection.len()));
            ui.horizontal(|ui|{
                if ui.button("Select matches").clicked(){self.commit_draft();self.selection=matches.iter().filter(|n|!n.hidden&&!n.has_locked()).map(|n|n.id.clone()).collect();self.sync_browser_selection();}
                if ui.button("Invert selection").clicked(){self.commit_draft();self.selection=self.history.project.parts.iter().filter(|n|!n.hidden&&!n.has_locked()&&!self.selection.contains(&n.id)).map(|n|n.id.clone()).collect();self.sync_browser_selection();}
                if ui.button("Clear").clicked(){self.commit_draft();self.selection.clear();self.sync_browser_selection();}
            });
            ui.small("Search selects matching top-level objects; hidden and locked objects are excluded from bulk selection. Shift-click adds/removes.");
            ui.separator();ui.label("Rename selection (multiple objects get numbered suffixes)");ui.text_edit_singleline(&mut self.tools.rename);
            if ui.button("Rename selected").clicked(){self.commit_draft();let name=self.tools.rename.clone();self.operation(|p,s|{modeling::rename_selection(p,s,&name)?;Ok(None)});}
            ui.separator();
            for n in matches{ui.push_id(&n.id,|ui|{
                let label=format!("{} · {}{}{}",n.name,n.kind(),if n.hidden{" · hidden"}else{""},if n.has_locked(){" · locked"}else{""});
                if ui.selectable_label(self.selection.contains(&n.id),label).clicked(){self.select(n.id.clone(),ctx.input(|i|i.modifiers.shift));}
                if matches!(n.shape,Shape::Group{..}){tree(ui,&n);}
            });}
            ui.small("Select a group and use F2 for component editing.");
        });
        self.tools.scene = open;
    }
}
