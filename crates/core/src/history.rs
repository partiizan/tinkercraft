use crate::Project;
use anyhow::Result;
#[derive(Clone)]
pub struct History {
    pub project: Project,
    past: Vec<Project>,
    future: Vec<Project>,
    saved: Option<Project>,
}
impl History {
    pub fn new(p: Project, saved: bool) -> Self {
        Self {
            saved: if saved { Some(p.clone()) } else { None },
            project: p,
            past: vec![],
            future: vec![],
        }
    }
    pub fn dirty(&self) -> bool {
        self.saved.as_ref() != Some(&self.project)
    }
    pub fn mark_saved(&mut self) {
        self.saved = Some(self.project.clone());
    }
    pub fn commit(&mut self, p: Project) -> Result<bool> {
        p.validate()?;
        crate::operations::protect_locked(&self.project, &p)?;
        if p == self.project {
            return Ok(false);
        }
        self.past.push(std::mem::replace(&mut self.project, p));
        if self.past.len() > 100 {
            self.past.remove(0);
        }
        self.future.clear();
        Ok(true)
    }
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
    pub fn undo(&mut self) -> bool {
        if let Some(p) = self.past.pop() {
            self.future.push(std::mem::replace(&mut self.project, p));
            true
        } else {
            false
        }
    }
    pub fn redo(&mut self) -> bool {
        if let Some(p) = self.future.pop() {
            self.past.push(std::mem::replace(&mut self.project, p));
            true
        } else {
            false
        }
    }
}
