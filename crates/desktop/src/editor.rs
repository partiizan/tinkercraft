use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use glam::{DVec2, DVec3};
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};
use tinkercraft_core::{
    geometry::{self, Bounds, SourceMesh},
    operations::{self, Selection},
    storage::{self, DocumentFile},
    viewport::{self, Camera},
    *,
};
struct Computed {
    revision: u64,
    sources: Vec<SourceMesh>,
    result: Result<Mesh, String>,
}
#[derive(Clone)]
enum Pending {
    New,
    Demo,
    Open(PathBuf),
    Close,
}
#[derive(Clone, Copy, PartialEq)]
enum Tool {
    Move,
    Rotate,
    Resize,
}
struct Drag {
    node: Node,
    axis: usize,
    start: Pos2,
    screen_axis: Vec2,
    world_length: f64,
    tool: Tool,
    sources: Vec<SourceMesh>,
}
pub struct Editor {
    export_requested: bool,
    import_path: Option<PathBuf>,
    import_scale: f64,
    import_job: Option<mpsc::Receiver<Result<Node, String>>>,
    import_node: Option<Node>,
    import_error: String,
    history: History,
    file: DocumentFile,
    selection: Selection,
    draft: Option<Node>,
    name: String,
    sender: mpsc::Sender<(u64, Project)>,
    receiver: mpsc::Receiver<Computed>,
    revision: u64,
    ready: u64,
    sources: Vec<SourceMesh>,
    result: Option<Mesh>,
    status: String,
    camera: Camera,
    preferences_path: Option<PathBuf>,
    last_preferences: preferences::Preferences,
    preferences_due: Option<Instant>,
    preferences_error: bool,
    preferences_status: String,
    preferences_open: bool,
    remember_camera: bool,
    preview: bool,
    tool: Tool,
    move_snap: f64,
    angle_snap: f64,
    drag: Option<Drag>,
    frame: Option<viewport::Frame>,
    texture: Option<egui::TextureHandle>,
    redraw: bool,
    pending: Option<Pending>,
    allow_close: bool,
    recovery_path: Option<PathBuf>,
    recovery: Option<Project>,
    recovery_error: bool,
    recovery_due: Option<Instant>,
    recovery_status: String,
}
impl Editor {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let (tx, rx) = mpsc::channel::<(u64, Project)>();
        let (out_tx, out_rx) = mpsc::channel();
        let ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || {
            while let Ok(mut job) = rx.recv() {
                while let Ok(next) = rx.try_recv() {
                    job = next
                }
                let (revision, p) = job;
                let computed = std::panic::catch_unwind(|| {
                    let sources = geometry::sources(&p);
                    let result = evaluate(&p).map_err(|e| e.to_string());
                    Computed {
                        revision,
                        sources,
                        result,
                    }
                })
                .unwrap_or_else(|_| Computed {
                    revision,
                    sources: vec![],
                    result: Err("Geometry engine failed on this model".into()),
                });
                if out_tx.send(computed).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });
        let recovery_path = directories::ProjectDirs::from("com", "TinkerCraft", "TinkerCraft")
            .map(|d| d.data_local_dir().join("recovery.tinkercraft"));
        let mut recovery = None;
        let mut recovery_error = false;
        let mut recovery_status = String::new();
        if let Some(path) = &recovery_path
            && path.exists()
        {
            match storage::read(path).and_then(|b| Project::parse(&b)) {
                Ok(p) => recovery = Some(p),
                Err(e) => {
                    recovery_error = true;
                    recovery_status = format!("Recovery retained; could not read it: {e}");
                }
            }
        }
        let preferences_path = directories::ProjectDirs::from("com", "TinkerCraft", "TinkerCraft")
            .map(|d| d.config_dir().join("preferences.json"));
        let mut preferences = preferences::Preferences::default();
        let mut preferences_error = false;
        let mut preferences_status = String::new();
        if let Some(path) = &preferences_path
            && path.exists()
        {
            match preferences::Preferences::load(path) {
                Ok(p) => preferences = p,
                Err(e) => {
                    preferences_error = true;
                    preferences_status = format!(
                        "Preferences retained but unreadable: {e}. Reset defaults to replace them."
                    );
                }
            }
        }
        let p = Project::demo();
        let mut app = Self {
            export_requested: false,
            import_path: None,
            import_scale: 1.,
            import_job: None,
            import_node: None,
            import_error: String::new(),
            history: History::new(p.clone(), false),
            file: DocumentFile::default(),
            selection: Selection::from([p.parts[0].id.clone()]),
            draft: None,
            name: p.name.clone(),
            sender: tx,
            receiver: out_rx,
            revision: 0,
            ready: u64::MAX,
            sources: vec![],
            result: None,
            status: "Starting geometry engine…".into(),
            camera: Camera::default(),
            preferences_path,
            last_preferences: preferences.clone(),
            preferences_due: None,
            preferences_error,
            preferences_status,
            preferences_open: false,
            remember_camera: preferences.remember_camera,
            preview: false,
            tool: Tool::Move,
            move_snap: 1.,
            angle_snap: 15.,
            drag: None,
            frame: None,
            texture: None,
            redraw: true,
            pending: None,
            allow_close: false,
            recovery_path,
            recovery,
            recovery_error,
            recovery_due: None,
            recovery_status,
        };
        app.refresh();
        app.fit(1.4);
        app.move_snap = preferences.move_snap;
        app.angle_snap = preferences.angle_snap;
        app.tool = [Tool::Move, Tool::Rotate, Tool::Resize][preferences.tool as usize];
        app.camera.orthographic = preferences.orthographic;
        if preferences.remember_camera
            && app.preferences_path.as_ref().is_some_and(|p| p.exists())
            && !app.preferences_error
        {
            app.camera.target = DVec3::from(preferences.target);
            app.camera.yaw = preferences.yaw;
            app.camera.pitch = preferences.pitch;
            app.camera.distance = preferences.distance;
        }
        // Optional launch-file entry point; the same review/validation flow as the picker.
        if let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) {
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("stl"))
            {
                app.import_path = Some(path);
            } else {
                app.status =
                    "Launch-file support accepts an STL path; use Open for projects".into();
            }
        }
        app
    }
    fn refresh(&mut self) {
        self.selection
            .retain(|id| self.history.project.parts.iter().any(|p| &p.id == id));
        if self.selection.is_empty()
            && let Some(p) = self
                .history
                .project
                .parts
                .iter()
                .find(|n| !n.hidden && !n.has_locked())
        {
            self.selection.insert(p.id.clone());
        }
        self.draft = if self.selection.len() == 1 {
            self.history
                .project
                .parts
                .iter()
                .find(|p| self.selection.contains(&p.id))
                .cloned()
        } else {
            None
        };
        self.name = self.history.project.name.clone();
        self.revision += 1;
        self.result = None;
        self.preview = false;
        self.status = "Calculating…".into();
        let _ = self
            .sender
            .send((self.revision, self.history.project.clone()));
        self.redraw = true;
        if self.recovery.is_none() && !self.recovery_error {
            self.recovery_due = Some(Instant::now() + Duration::from_millis(700));
        }
    }
    fn commit(&mut self, p: Project) {
        match self.history.commit(p) {
            Ok(true) => self.refresh(),
            Ok(false) => {}
            Err(e) => {
                self.refresh();
                self.status = e.to_string();
            }
        }
    }
    fn operation(
        &mut self,
        f: impl FnOnce(&mut Project, &Selection) -> anyhow::Result<Option<Selection>>,
    ) {
        let mut p = self.history.project.clone();
        if p.parts
            .iter()
            .any(|n| self.selection.contains(&n.id) && (n.has_locked() || n.hidden))
        {
            self.status = "Show and unlock the selected objects before editing".into();
            return;
        }
        match f(&mut p, &self.selection) {
            Ok(ids) => {
                if let Some(ids) = ids {
                    self.selection = ids
                }
                self.commit(p);
            }
            Err(e) => self.status = e.to_string(),
        }
    }
    fn group_selection(&mut self) {
        self.commit_draft();
        let previous: Selection = self
            .history
            .project
            .parts
            .iter()
            .map(|n| n.id.clone())
            .collect();
        self.operation(|p, s| Ok(Some(Selection::from([operations::group(p, s)?]))));
        // Switch only after a group was successfully committed, never on rejected edits.
        if self.history.project.parts.iter().any(|n| {
            self.selection.contains(&n.id)
                && !previous.contains(&n.id)
                && matches!(n.shape, Shape::Group { .. })
        }) {
            self.preview = true;
            self.redraw = true;
        }
    }
    fn select(&mut self, id: String, toggle: bool) {
        self.commit_draft();
        if !toggle {
            self.selection.clear();
        }
        if !self.selection.insert(id.clone()) && toggle {
            self.selection.remove(&id);
        }
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
    fn fit(&mut self, aspect: f64) {
        let mut b = Bounds::default();
        for n in self
            .history
            .project
            .parts
            .iter()
            .filter(|n| self.preview || !n.hidden)
        {
            b.merge(geometry::bounds(n));
        }
        self.camera.fit(b, aspect);
        self.redraw = true;
    }
    fn save(&mut self, save_as: bool) -> bool {
        self.commit_draft();
        let path = if save_as || self.file.path.is_none() {
            let suggested = format!(
                "{}.tinkercraft",
                self.history.project.name.replace(['/', '\\'], "_")
            );
            match rfd::FileDialog::new()
                .add_filter("TinkerCraft project", &["tinkercraft"])
                .set_file_name(suggested)
                .save_file()
            {
                Some(p) => Some(p),
                None => return false,
            }
        } else {
            None
        };
        match self.file.save(&self.history.project, path) {
            Ok(()) => {
                self.history.mark_saved();
                self.status = "Project saved".into();
                self.flush_recovery();
                true
            }
            Err(e) => {
                self.status = e.to_string();
                false
            }
        }
    }
    fn commit_draft(&mut self) {
        if let Some(n) = self.draft.clone() {
            let mut p = self.history.project.clone();
            if let Some(old) = p.parts.iter_mut().find(|p| p.id == n.id) {
                *old = n;
                self.commit(p);
            }
        }
        if self.name != self.history.project.name {
            let mut p = self.history.project.clone();
            p.name = self.name.clone();
            self.commit(p);
        }
    }
    fn request(&mut self, pending: Pending) {
        self.commit_draft();
        if self.history.dirty() {
            self.pending = Some(pending)
        } else {
            self.perform(pending)
        }
    }
    fn perform(&mut self, pending: Pending) {
        self.export_requested = false;
        let opened = match pending {
            Pending::Close => {
                self.allow_close = true;
                self.clear_recovery();
                return;
            }
            Pending::Open(path) => match DocumentFile::open(path) {
                Ok((file, p)) => Some((file, p, true)),
                Err(e) => {
                    self.status = e.to_string();
                    None
                }
            },
            Pending::New => Some((DocumentFile::default(), Project::default(), true)),
            Pending::Demo => Some((DocumentFile::default(), Project::demo(), false)),
        };
        if let Some((file, p, saved)) = opened {
            self.file = file;
            self.history = History::new(p, saved);
            self.selection.clear();
            self.refresh();
            self.fit(1.4);
        }
    }
    fn open(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("TinkerCraft project", &["tinkercraft", "shapeworkshop"])
            .pick_file()
        {
            match DocumentFile::open(path.clone()) {
                Ok(_) => self.request(Pending::Open(path)),
                Err(e) => self.status = e.to_string(),
            }
        }
    }
    fn clear_recovery(&mut self) {
        if self.recovery_error {
            return;
        }
        if let Some(p) = &self.recovery_path
            && p.exists()
            && let Err(e) = std::fs::remove_file(p)
        {
            self.recovery_status = format!("Cannot clear recovery: {e}");
            return;
        }
        self.recovery_status = "All changes saved".into();
        self.recovery_due = None;
    }
    fn flush_recovery(&mut self) {
        if self.recovery.is_some() || self.recovery_error {
            return;
        }
        self.recovery_due = None;
        if !self.history.dirty() {
            self.clear_recovery();
            return;
        }
        if let Some(path) = &self.recovery_path {
            let result = (|| -> anyhow::Result<()> {
                std::fs::create_dir_all(path.parent().unwrap())?;
                storage::atomic_write(path, &self.history.project.bytes()?)?;
                Ok(())
            })();
            self.recovery_status = match result {
                Ok(()) => "Recovery snapshot saved".into(),
                Err(e) => format!("Recovery unavailable: {e}"),
            };
        }
    }
    fn export(&mut self) {
        self.commit_draft();
        self.export_requested = true;
    }
    fn write_export(&mut self) {
        self.export_requested = false;
        if self.ready != self.revision {
            return;
        }
        let Some(m) = &self.result else { return };
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("STL mesh", &["stl"])
            .set_file_name("tinkercraft.stl")
            .save_file()
        {
            match m.stl().and_then(|b| storage::atomic_write(&p, &b)) {
                Ok(()) => self.status = "STL exported in millimeter coordinates".into(),
                Err(e) => self.status = e.to_string(),
            }
        }
    }
    fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(6.);
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("TinkerCraft").color(Color32::from_rgb(117, 222, 188)),
                );
                ui.label(format!("Native Rust · {}", env!("CARGO_PKG_VERSION")));
                ui.separator();
                if ui.button("Preferences").clicked() {
                    self.preferences_open = true;
                }
                if ui.button("New").clicked() {
                    self.request(Pending::New)
                }
                if ui.button("Open").clicked() {
                    self.open()
                }
                if ui.button("Import STL").clicked() {
                    self.commit_draft();
                    self.import_path = rfd::FileDialog::new()
                        .add_filter("STL mesh", &["stl"])
                        .pick_file();
                    self.import_node = None;
                    self.import_error.clear();
                }
                if ui.button("Save").clicked() {
                    self.save(false);
                }
                if ui.button("Save as").clicked() {
                    self.save(true);
                }
                if ui
                    .add_enabled(
                        self.ready == self.revision && self.result.is_some() && self.drag.is_none(),
                        egui::Button::new("Export STL"),
                    )
                    .clicked()
                {
                    self.export()
                }
                ui.separator();
                ui.label(if self.history.dirty() {
                    "Unsaved changes"
                } else {
                    "Saved locally"
                });
            });
            ui.add_space(6.);
        });
    }
    fn import_modal(&mut self, ctx: &egui::Context) {
        if self.pending.is_some() || self.recovery.is_some() {
            return;
        }
        let Some(path) = self.import_path.clone() else {
            return;
        };
        if let Some(job) = &self.import_job {
            match job.try_recv() {
                Ok(result) => {
                    self.import_job = None;
                    match result {
                        Ok(n) => self.import_node = Some(n),
                        Err(e) => self.import_error = e,
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.import_job = None;
                    self.import_error = "Import worker stopped".into();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        egui::Modal::new(egui::Id::new("import_stl")).show(ctx, |ui| {
            ui.heading("Import STL");
            ui.label(path.file_name().unwrap_or_default().to_string_lossy());
            ui.label("STL has no units. Choose the source units before importing.");
            if self.import_job.is_some() { ui.spinner(); ui.label("Checking mesh topology and solid geometry…"); ctx.request_repaint_after(Duration::from_millis(100)); }
            else if let Some(n) = self.import_node.clone() {
                let size=n.size().unwrap();
                if let Shape::Mesh { triangles, .. }=&n.shape { ui.label(format!("{} triangles · {:.2} × {:.2} × {:.2} mm",triangles.len(),size[0],size[1],size[2])); }
                ui.label("Centered at X40/Y15 and placed on the workplane.");
                if ui.button("Add to project").clicked() {
                    let mut p=self.history.project.clone(); p.parts.push(n.clone());
                    match p.validate().and_then(|_| p.bytes()) {
                        Ok(_) => { self.selection=Selection::from([n.id]); self.commit(p); self.preview=false; self.fit(1.4); self.import_path=None; self.import_node=None; },
                        Err(e) => self.import_error=e.to_string()
                    }
                }
                if ui.button("Change units").clicked() { self.import_node=None; }
            } else {
                ui.horizontal(|ui| { for (scale,label) in [(1.,"Millimeters"),(25.4,"Inches"),(10.,"Centimeters")] { ui.selectable_value(&mut self.import_scale,scale,label); } });
                ui.small("Maximum 10 MB / 20,000 triangles. Closed meshes only; no automatic hole repair.");
                if ui.button("Validate mesh").clicked() {
                    self.import_error.clear();
                    let scale=self.import_scale; let (tx,rx)=mpsc::channel(); self.import_job=Some(rx); let ctx=ctx.clone();
                    std::thread::spawn(move || {
                        let result=std::panic::catch_unwind(|| stl::read(&path).and_then(|b|stl::import(&b,&path.file_stem().unwrap_or_default().to_string_lossy(),scale)))
                            .map_err(|_| "Geometry engine failed while importing".to_string()).and_then(|r|r.map_err(|e|e.to_string()));
                        let _=tx.send(result); ctx.request_repaint();
                    });
                }
            }
            if !self.import_error.is_empty() { ui.colored_label(Color32::LIGHT_RED,&self.import_error); }
            if ui.button("Cancel").clicked() { self.import_path=None; self.import_job=None; self.import_node=None; }
        });
    }
    fn set_flags(&mut self, hidden: Option<bool>, locked: Option<bool>) {
        self.commit_draft();
        let mut p = self.history.project.clone();
        for n in p
            .parts
            .iter_mut()
            .filter(|n| self.selection.contains(&n.id))
        {
            if let Some(value) = hidden {
                n.hidden = value;
            }
            if let Some(value) = locked {
                n.set_locked_recursive(value);
            }
        }
        self.commit(p);
    }
    fn current_preferences(&self) -> preferences::Preferences {
        let c = if self.remember_camera {
            self.camera.clone()
        } else {
            Camera::default()
        };
        preferences::Preferences {
            version: 1,
            move_snap: self.move_snap,
            angle_snap: self.angle_snap,
            tool: match self.tool {
                Tool::Move => 0,
                Tool::Rotate => 1,
                Tool::Resize => 2,
            },
            orthographic: self.camera.orthographic,
            remember_camera: self.remember_camera,
            target: c.target.to_array(),
            yaw: c.yaw,
            pitch: c.pitch,
            distance: c.distance,
        }
    }
    fn sync_preferences(&mut self) {
        let p = self.current_preferences();
        if p != self.last_preferences {
            self.last_preferences = p;
            self.preferences_due = Some(Instant::now() + Duration::from_millis(500));
        }
        if self.preferences_due.is_some_and(|t| Instant::now() >= t) {
            self.save_preferences();
        }
    }
    fn save_preferences(&mut self) {
        self.preferences_due = None;
        if self.preferences_error {
            return;
        }
        if let Some(path) = &self.preferences_path {
            match self.current_preferences().save(path) {
                Ok(()) => self.preferences_status = "Preferences saved locally".into(),
                Err(e) => self.preferences_status = format!("Preferences could not be saved: {e}"),
            }
        }
    }
    fn preferences_modal(&mut self, ctx: &egui::Context) {
        if !self.preferences_open
            || self.pending.is_some()
            || self.recovery.is_some()
            || self.import_path.is_some()
        {
            return;
        }
        egui::Modal::new(egui::Id::new("preferences")).show(ctx, |ui| {
            ui.heading("Preferences");
            ui.label("Tool, snapping and projection are saved automatically.");
            ui.checkbox(
                &mut self.remember_camera,
                "Remember camera between sessions",
            );
            ui.label("Project geometry, hidden and locked states are saved in the project.");
            if !self.preferences_status.is_empty() {
                ui.label(&self.preferences_status);
            }
            if ui.button("Reset defaults").clicked() {
                self.move_snap = 1.;
                self.angle_snap = 15.;
                self.tool = Tool::Move;
                self.camera = Camera::default();
                self.remember_camera = true;
                self.preferences_error = false;
                self.redraw = true;
                self.save_preferences();
            }
            if ui.button("Done").clicked() {
                self.preferences_open = false;
                self.save_preferences();
            }
        });
    }
    fn left(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("objects").exact_width(215.).show(ctx,|ui|{egui::ScrollArea::vertical().show(ui,|ui|{ui.heading("Project");let response=ui.text_edit_singleline(&mut self.name);if response.lost_focus(){self.commit_draft();}ui.horizontal(|ui|{if ui.add_enabled(self.history.can_undo(),egui::Button::new("Undo")).clicked(){self.history.undo();self.refresh();}
if ui.add_enabled(self.history.can_redo(),egui::Button::new("Redo")).clicked(){self.history.redo();self.refresh();}});ui.separator();ui.heading("Add a shape");egui::Grid::new("palette").num_columns(2).show(ui,|ui|{for(i,kind)in["Box","Cylinder","Sphere","Cone","Wedge","Roof"].iter().enumerate(){if crate::palette::button(ui, kind).clicked(){let n=Node::primitive(kind);let id=n.id.clone();let mut p=self.history.project.clone();p.parts.push(n);self.selection=Selection::from([id]);self.commit(p);}
if i%2==1{ui.end_row();}}});ui.separator();ui.heading(format!("Objects · {}",self.history.project.parts.len()));if ui.button("Select all").clicked(){self.selection=self.history.project.parts.iter().filter(|n|!n.hidden&&!n.has_locked()).map(|p|p.id.clone()).collect();self.draft=None;self.redraw=true;}ui.horizontal(|ui|{if ui.add_enabled(self.selection.len()>=2,egui::Button::new("Group")).clicked(){self.group_selection();}
if ui.add_enabled(self.selection.len()==1,egui::Button::new("Ungroup")).clicked(){self.operation(|p,s|Ok(Some(operations::ungroup(p,s.iter().next().unwrap())?)));}});
 for n in self.history.project.parts.clone(){
    let label=format!("{} {}{}{}",if n.hole{"H"}else{"S"},n.name,if n.hidden{" [hidden]"}else{""},if n.has_locked(){" [locked]"}else{""});
    if ui.add_sized([ui.available_width(),18.],egui::Button::new(label).selected(self.selection.contains(&n.id))).clicked(){self.select(n.id,ctx.input(|i|i.modifiers.shift));}
 }
 ui.horizontal(|ui| {
    let selected:Vec<_>=self.history.project.parts.iter().filter(|n|self.selection.contains(&n.id)).collect();
    let hidden=!selected.is_empty()&&selected.iter().all(|n|n.hidden);
    let locked=!selected.is_empty()&&selected.iter().all(|n|n.has_locked());
    if ui.add_enabled(!selected.is_empty(),egui::Button::new(if hidden{"Show"}else{"Hide"})).clicked(){self.set_flags(Some(!hidden),None);}
    if ui.add_enabled(!self.selection.is_empty(),egui::Button::new(if locked{"Unlock"}else{"Lock"})).clicked(){self.set_flags(None,Some(!locked));}
    if ui.button("Show all").clicked(){let mut p=self.history.project.clone();fn show(n:&mut Node){n.hidden=false;if let Shape::Group{children,..}=&mut n.shape{for c in children{show(c);}}}for n in &mut p.parts{show(n);}self.commit(p);}
 });
 ui.small("Hide affects editing view only; all objects are included in cuts/export.");

 ui.separator();if ui.add_enabled(self.result.is_some(),egui::Button::new(if self.preview{"Edit shapes"}else{"Show cut result"})).clicked(){self.preview = !self.preview;self.redraw=true;}ui.label(egui::RichText::new("Each group unions its solids and subtracts its holes.").small());if ui.button("Load plate example").clicked(){self.request(Pending::Demo)}ui.separator();ui.label("Drag background: orbit\nRight-drag: pan\nScroll: zoom\nShift-click: multiselect\nW / E / R: move / rotate / resize\nEsc: cancel handle drag");});});
    }
    fn inspector(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector").exact_width(245.).show(ctx,|ui|{egui::ScrollArea::vertical().show(ui,|ui|{
let protected=self.history.project.parts.iter().any(|n|self.selection.contains(&n.id)&&(n.has_locked()||n.hidden));
if protected {ui.label("Hidden or locked: show/unlock to edit.");}
ui.add_enabled_ui(!protected,|ui|{let mut commit=false;if let Some(n)=&mut self.draft{ui.heading(&n.name);ui.horizontal(|ui|{if ui.selectable_label(!n.hole,"Solid").clicked(){n.hole=false;commit=true}
if ui.selectable_label(n.hole,"Hole").clicked(){n.hole=true;commit=true}});ui.separator();if let Shape::Group{scale,children}=&mut n.shape{ui.label(format!("{} components · ungroup to edit",children.len()));ui.label("Uniform scale");commit|=number(ui,scale,0.01,0.00001..=100000.);}else{ui.label("Dimensions · mm");let size=n.size().unwrap();for(axis,v)in size.iter().enumerate(){if n.kind()=="Sphere"&&axis>0{continue}
if matches!(n.shape,Shape::Cylinder{..}|Shape::Cone{..})&&axis==1{continue}let label=if matches!(n.shape,Shape::Box{..}|Shape::Wedge{..}|Shape::Roof{..}|Shape::Mesh{..}){["Width X","Depth Y","Height Z"][axis]}else if axis==0{"Diameter"}else{"Height Z"};let mut value=*v;ui.horizontal(|ui|{ui.label(label);if number(ui,&mut value,0.1,0.01..=1000.){commit=true} });n.set_dimension(axis,value);}}
 ui.separator();ui.label("Position · mm");for axis in 0..3{ui.horizontal(|ui|{ui.label(["X","Y","Z"][axis]);commit|=number(ui,&mut n.position[axis],0.1,-10000. ..=10000.);});}ui.separator();ui.label("Rotation · degrees");for axis in 0..3{ui.horizontal(|ui|{ui.label(["X","Y","Z"][axis]);commit|=number(ui,&mut n.rotation[axis],1.,-3600. ..=3600.);});}
 }else{ui.heading(format!("{} selected",self.selection.len()));ui.label("Group objects to move or rotate them together.");}
if commit{self.commit_draft();}ui.separator();ui.horizontal(|ui|{if ui.button("Duplicate").clicked(){self.operation(|p,s|Ok(Some(operations::duplicate(p,s))));}
if ui.button("Delete").clicked(){self.operation(|p,s|{p.parts.retain(|n|!s.contains(&n.id));Ok(None)});}});if ui.button("Drop to workplane").clicked(){self.operation(|p,s|{operations::drop_to_plane(p,s)?;Ok(None)});}ui.separator();ui.heading("Align selection");ui.small("Combined source bounds, before cuts");for axis in 0..3{ui.horizontal(|ui|{ui.label(["X","Y","Z"][axis]);for(mode,label)in[(-1,"Min"),(0,"Center"),(1,"Max")]{if ui.add_enabled(self.selection.len()>1,egui::Button::new(label)).clicked(){self.operation(|p,s|{operations::align(p,s,axis,mode)?;Ok(None)});}}});}ui.separator();ui.heading("Mirror selection");ui.small("In place about the selection center");ui.horizontal(|ui|{for(axis,label)in["X","Y","Z"].iter().enumerate(){if ui.button(*label).clicked(){self.operation(|p,s|{operations::mirror(p,s,axis)?;Ok(None)});}}});ui.separator();ui.small("Z up · millimeters\nShift-click selects multiple objects. Ungrouping exposes holes to sibling scope. Hole groups must be made Solid first.");});});});
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if self.drag.is_some() {
            if ctx.input(|i| i.key_pressed(egui::Key::Escape) || !i.focused) {
                self.cancel_drag();
            }
            return;
        }
        let cmd = egui::Modifiers::COMMAND;
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers { shift: true, ..cmd }, egui::Key::S)) {
            self.save(true);
        } else if ctx.input_mut(|i| i.consume_key(cmd, egui::Key::S)) {
            self.save(false);
        }
        if ctx.input_mut(|i| i.consume_key(cmd, egui::Key::O)) {
            self.open();
        }
        if ctx.wants_keyboard_input() {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers { shift: true, ..cmd }, egui::Key::G)) {
            self.operation(|p, s| {
                let id = s
                    .iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Select a group"))?;
                Ok(Some(operations::ungroup(p, id)?))
            });
        } else if ctx.input_mut(|i| i.consume_key(cmd, egui::Key::G)) {
            self.group_selection();
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers { shift: true, ..cmd }, egui::Key::Z)) {
            if self.history.redo() {
                self.refresh()
            }
        } else if ctx.input_mut(|i| i.consume_key(cmd, egui::Key::Z)) && self.history.undo() {
            self.refresh()
        }
        if ctx.input_mut(|i| i.consume_key(cmd, egui::Key::A)) {
            self.selection = self
                .history
                .project
                .parts
                .iter()
                .filter(|p| !p.hidden && !p.has_locked())
                .map(|p| p.id.clone())
                .collect();
            self.draft = None;
            self.redraw = true;
        }
        if ctx.input_mut(|i| i.consume_key(cmd, egui::Key::D)) {
            self.operation(|p, s| Ok(Some(operations::duplicate(p, s))));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            self.operation(|p, s| {
                p.parts.retain(|n| !s.contains(&n.id));
                Ok(None)
            });
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F)) {
            self.fit_selection(1.4);
        }
        if ctx.input(|i| !i.modifiers.command && !i.modifiers.alt) {
            let step = if self.move_snap > 0. {
                self.move_snap
            } else {
                1.
            } * if ctx.input(|i| i.modifiers.shift) {
                10.
            } else {
                1.
            };
            for (key, axis, direction) in [
                (egui::Key::ArrowLeft, 0, -1.),
                (egui::Key::ArrowRight, 0, 1.),
                (egui::Key::ArrowDown, 1, -1.),
                (egui::Key::ArrowUp, 1, 1.),
                (egui::Key::PageDown, 2, -1.),
                (egui::Key::PageUp, 2, 1.),
            ] {
                if ctx.input(|i| i.key_pressed(key)) {
                    let mut delta = [0.; 3];
                    delta[axis] = step * direction;
                    self.operation(|p, s| {
                        operations::translate(p, s, delta)?;
                        Ok(None)
                    });
                }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selection.clear();
            self.draft = None;
            self.redraw = true;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::W)) {
            self.tool = Tool::Move
        }
        if ctx.input(|i| i.key_pressed(egui::Key::E)) {
            self.tool = Tool::Rotate
        }
        if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            self.tool = Tool::Resize;
        }
    }
    fn fit_selection(&mut self, aspect: f64) {
        let mut b = Bounds::default();
        for n in self
            .history
            .project
            .parts
            .iter()
            .filter(|n| self.selection.contains(&n.id) && (!n.hidden || self.preview))
        {
            b.merge(geometry::bounds(n));
        }
        self.camera.fit(b, aspect);
        self.redraw = true;
    }
    fn cancel_drag(&mut self) {
        if let Some(d) = self.drag.take() {
            self.sources = d.sources;
            self.draft = Some(d.node);
            self.redraw = true;
        }
    }
    fn viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                for view in ["Iso", "Top", "Front", "Right"] {
                    if ui.button(view).clicked() {
                        self.camera.view(view);
                        self.redraw = true;
                    }
                }
                if ui.button("Fit all").clicked() {
                    self.fit(
                        (ui.max_rect().width() / (ui.max_rect().height() - 65.).max(1.)) as f64,
                    );
                }
                if ui.checkbox(&mut self.camera.orthographic, "Orthographic").changed() { self.redraw=true; }
                if ui.button("Fit selection (F)").clicked() {self.fit_selection((ui.max_rect().width()/(ui.max_rect().height()-65.).max(1.)) as f64);}
            });
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tool, Tool::Move, "Move (W)");
                ui.selectable_value(&mut self.tool, Tool::Rotate, "Rotate (E)");
                ui.selectable_value(&mut self.tool, Tool::Resize, "Resize (R)");
                egui::ComboBox::from_id_salt("move_snap")
                    .selected_text(format!("Snap {} mm", self.move_snap))
                    .show_ui(ui, |ui| {
                        for snap in [0., 0.1, 0.5, 1., 5.] {
                            ui.selectable_value(
                                &mut self.move_snap,
                                snap,
                                if snap == 0. {
                                    "Off".into()
                                } else {
                                    format!("{snap} mm")
                                },
                            );
                        }
                    });
                egui::ComboBox::from_id_salt("angle_snap")
                    .selected_text(format!("Angle {}°", self.angle_snap))
                    .show_ui(ui, |ui| {
                        for snap in [0., 5., 15., 45., 90.] {
                            ui.selectable_value(&mut self.angle_snap, snap, format!("{snap}°"));
                        }
                    });
            });
            if self.tool == Tool::Resize {
                ui.small("Resize in local axes about the center/pivot · groups scale uniformly · drag or enter exact dimensions");
            }
            let available = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());
            let width = rect.width().clamp(1., 900.) as usize;
            let height = ((width as f32 * rect.height() / rect.width()).clamp(1., 900.)) as usize;
            if self
                .frame
                .as_ref()
                .is_none_or(|f| f.width != width || f.height != height)
            {
                self.redraw = true;
            }
            let local = |p: Pos2| {
                DVec2::new(
                    (p.x - rect.left()) as f64 / rect.width() as f64 * width as f64,
                    (p.y - rect.top()) as f64 / rect.height() as f64 * height as f64,
                )
            };
            let to_screen = |p: [f64; 3]| {
                Pos2::new(
                    rect.left() + p[0] as f32 / width as f32 * rect.width(),
                    rect.top() + p[1] as f32 / height as f32 * rect.height(),
                )
            };
            let single =
                if self.selection.len() == 1 && !self.preview && self.ready == self.revision {
                    if self.drag.is_some() {
                        self.draft.clone()
                    } else {
                        self.history
                            .project
                            .parts
                            .iter()
                            .find(|p| self.selection.contains(&p.id) && !p.hidden && !p.has_locked())
                            .cloned()
                    }
                } else {
                    None
                };
            let mut handles = vec![];
            if let Some(n) = &single {
                let center = DVec3::from(n.position);
                let length = self.camera.units_per_pixel(height as f64) * 65.;
                if let Some(origin) = self.camera.project(center, width, height) {
                    for axis in 0..3 {
                        let mut endpoint = center;
                        if self.tool == Tool::Resize {
                            let mut local=DVec3::ZERO; local[axis]=length;
                            endpoint += quaternion(n.rotation)*local;
                        } else { endpoint[axis] += length; }
                        if let Some(end) = self.camera.project(endpoint, width, height) {
                            handles.push((axis, to_screen(origin), to_screen(end), length));
                        }
                    }
                }
            }
            if response.drag_started_by(egui::PointerButton::Primary)
                && let Some(start) = ctx.input(|i| i.pointer.press_origin())
                    && let Some(n) = &single {
                        let hit = handles
                            .iter()
                            .filter(|(_, a, b, _)| a.distance(*b) > 8.)
                            .min_by(|(_, a, b, _), (_, c, d, _)| {
                                segment_distance(start, *a, *b)
                                    .total_cmp(&segment_distance(start, *c, *d))
                            })
                            .filter(|(_, a, b, _)| segment_distance(start, *a, *b) < 10.);
                        if let Some((axis, a, b, length)) = hit {
                            self.drag = Some(Drag {
                                node: n.clone(),
                                axis: *axis,
                                start,
                                screen_axis: *b - *a,
                                world_length: *length,
                                tool: self.tool,
                                sources: self.sources.clone(),
                            });
                        }
                    }
            if let Some(d) = &self.drag {
                if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    let delta = pos - d.start;
                    let mut n = d.node.clone();
                    if d.tool == Tool::Rotate {
                        let degrees = (delta.x - delta.y) as f64;
                        let degrees = snap(degrees, self.angle_snap);
                        operations::world_rotation(&mut n, d.axis, degrees);
                    } else {
                        let amount = delta.dot(d.screen_axis) / d.screen_axis.length_sq().max(1.)
                            * d.world_length as f32;
                        if d.tool == Tool::Resize {
                            let size=operations::local_dimensions(&d.node)[d.axis];
                            let dimension=snap(size+2.*amount as f64,self.move_snap).max(0.01);
                            match operations::resize_node(&d.node,d.axis,dimension) {
                                Ok(resized)=>n=resized,
                                Err(e)=> { self.status=e.to_string(); n=self.draft.clone().unwrap_or(n); }
                            }
                        } else {
                            n.position[d.axis] = snap(d.node.position[d.axis] + amount as f64, self.move_snap);
                        }
                    }
                    let matrix = operations::resize_preview_matrix(&d.node, &n);
                    self.sources = d
                        .sources
                        .iter()
                        .map(|s| SourceMesh {
                            id: s.id.clone(),
                            hole: s.hole,
                            mesh: if s.id == n.id {
                                s.mesh.transformed(matrix)
                            } else {
                                s.mesh.clone()
                            },
                        })
                        .collect();
                    self.draft = Some(n);
                    self.redraw = true;
                }
                if response.drag_stopped() {
                    self.drag = None;
                    self.commit_draft();
                }
            } else {
                if response.dragged_by(egui::PointerButton::Primary) {
                    let delta = ctx.input(|i| i.pointer.delta());
                    self.camera.yaw -= delta.x as f64 * 0.008;
                    self.camera.pitch =
                        (self.camera.pitch + delta.y as f64 * 0.008).clamp(-1.56, 1.56);
                    self.redraw = true;
                }
                if response.dragged_by(egui::PointerButton::Secondary) {
                    let delta = ctx.input(|i| i.pointer.delta());
                    let (right, up, _) = self.camera.basis();
                    self.camera.target += (-right * delta.x as f64 + up * delta.y as f64)
                        * self.camera.units_per_pixel(rect.height() as f64);
                    self.redraw = true;
                }
                if response.clicked()
                    && let (Some(pos), Some(frame)) = (response.interact_pointer_pos(), &self.frame)
                    {
                        let q = local(pos);
                        if let Some(id) = frame.pick(q.x.max(0.) as usize, q.y.max(0.) as usize) {
                            self.select(id, ctx.input(|i| i.modifiers.shift));
                        }
                    }
            }
            if response.hovered() && self.drag.is_none() {
                let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0. {
                    self.camera.distance =
                        (self.camera.distance * (-scroll as f64 * 0.002).exp()).clamp(1., 20000.);
                    self.redraw = true;
                }
            }
            if self.redraw {
                let frame = viewport::render(
                    &self.camera,
                    width,
                    height,
                    &self.sources,
                    &self.selection,
                    if self.preview {
                        self.result.as_ref()
                    } else {
                        None
                    },
                );
                let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &frame.rgba);
                if let Some(texture) = &mut self.texture {
                    texture.set(image, egui::TextureOptions::LINEAR)
                } else {
                    self.texture =
                        Some(ctx.load_texture("viewport", image, egui::TextureOptions::LINEAR));
                }
                self.frame = Some(frame);
                self.redraw = false;
            }
            if let Some(texture) = &self.texture {
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1., 1.)),
                    Color32::WHITE,
                );
            }
            for (axis, a, b, _) in handles {
                if a.distance(b) <= 8. { continue; }
                let color = [
                    Color32::from_rgb(225, 78, 74),
                    Color32::from_rgb(81, 183, 100),
                    Color32::from_rgb(74, 135, 235),
                ][axis];
                if self.tool == Tool::Resize {
                    ui.painter().line_segment([a,b],Stroke::new(2_f32,color));
                    ui.painter().rect_filled(egui::Rect::from_center_size(b,Vec2::splat(12.)),2.,color);
                } else { ui.painter().arrow(a, b - a, Stroke::new(3_f32, color)); }
                ui.painter().text(
                    b,
                    egui::Align2::LEFT_CENTER,
                    if self.tool == Tool::Rotate {
                        ["X rotate", "Y rotate", "Z rotate"][axis]
                    } else if self.tool == Tool::Resize {
                        ["Size X", "Size Y", "Size Z"][axis]
                    } else {
                        ["X", "Y", "Z"][axis]
                    },
                    egui::FontId::proportional(13.),
                    color,
                );
            }
            ui.painter().text(
                rect.left_bottom() + Vec2::new(12., -14.),
                egui::Align2::LEFT_BOTTOM,
                if self.preview {
                    "BOOLEAN RESULT"
                } else {
                    "XY workplane · 10 mm grid · Z up"
                },
                egui::FontId::proportional(12.),
                Color32::from_gray(75),
            );
        });
    }
}
fn number(
    ui: &mut egui::Ui,
    v: &mut f64,
    speed: f64,
    range: std::ops::RangeInclusive<f64>,
) -> bool {
    let r = ui.add(
        egui::DragValue::new(v)
            .speed(speed)
            .range(range)
            .max_decimals(6),
    );
    r.drag_stopped() || r.lost_focus()
}
fn snap(v: f64, step: f64) -> f64 {
    if step > 0. {
        (v / step).round() * step
    } else {
        v
    }
}
fn segment_distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_sq().max(0.001)).clamp(0., 1.);
    p.distance(a + ab * t)
}
impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(c) = self.receiver.try_recv() {
            if c.revision != self.revision {
                continue;
            }
            self.ready = c.revision;
            self.sources = c.sources;
            match c.result {
                Ok(m) => {
                    self.status = format!(
                        "Ready · {} triangles · {:.1} mm³",
                        m.triangles.len(),
                        m.volume()
                    );
                    self.result = Some(m)
                }
                Err(e) => {
                    self.status = e;
                    self.result = None
                }
            }
            self.redraw = true;
        }
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.cancel_drag();
            self.request(Pending::Close);
        }
        if self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.recovery_due.is_some_and(|t| Instant::now() >= t) {
            self.flush_recovery();
        }
        if self.recovery_due.is_some() || self.preferences_due.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "{}{} — TinkerCraft",
            if self.history.dirty() { "* " } else { "" },
            self.history.project.name
        )));
        if self.pending.is_none()
            && self.recovery.is_none()
            && self.import_path.is_none()
            && !self.preferences_open
        {
            self.shortcuts(ctx);
        }
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.separator();
                ui.small(&self.recovery_status);
            });
        });
        // Disable editor interaction while a modal decision is pending.
        let blocked = self.pending.is_some() || self.recovery.is_some();
        if blocked {
            ctx.memory_mut(|m| {
                m.set_focus_lock_filter(egui::Id::new("modal"), egui::EventFilter::default())
            });
        }
        self.toolbar(ctx);
        self.left(ctx);
        self.inspector(ctx);
        self.viewport(ctx);
        self.import_modal(ctx);
        self.preferences_modal(ctx);
        if self.export_requested && self.ready == self.revision && self.pending.is_none() {
            self.write_export();
        }
        self.sync_preferences();
        if self.preferences_due.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
        if let Some(p) = self.recovery.clone() {
            egui::Modal::new(egui::Id::new("recovery")).show(ctx, |ui| {
                ui.heading("Recover your project?");
                ui.label(format!("An unsaved snapshot of “{}” is available.", p.name));
                ui.horizontal(|ui| {
                    if ui.button("Restore").clicked() {
                        self.recovery = None;
                        self.history = History::new(p.clone(), false);
                        self.file = DocumentFile::default();
                        self.selection.clear();
                        self.refresh();
                        self.fit(1.4);
                    }
                    if ui.button("Discard snapshot").clicked() {
                        self.recovery = None;
                        self.clear_recovery();
                    }
                });
            });
        }
        if let Some(pending) = self.pending.clone() {
            egui::Modal::new(egui::Id::new("unsaved")).show(ctx, |ui| {
                ui.heading("Save changes?");
                ui.label("Save this project before continuing, or discard its changes.");
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.pending = None;
                    }
                    if ui.button("Discard").clicked() {
                        self.pending = None;
                        self.perform(pending.clone());
                    }
                    if ui.button("Save").clicked() && self.save(false) {
                        self.pending = None;
                        self.perform(pending.clone());
                    }
                });
            });
        }
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_preferences();
        if !self.allow_close {
            self.flush_recovery();
        }
    }
}
