//! Local user preferences, separate from project content and undo history.
use crate::{document::Vec3, storage, viewport::Camera};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{io::Read, path::Path};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    Light,
    Dark,
    #[default]
    System,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub version: u32,
    pub appearance: Appearance,
    pub move_snap: f64,
    pub angle_snap: f64,
    pub tool: u8,
    pub orthographic: bool,
    pub remember_camera: bool,
    pub target: Vec3,
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
}
impl Default for Preferences {
    fn default() -> Self {
        let c = Camera::default();
        Self {
            version: 1,
            appearance: Appearance::System,
            move_snap: 1.,
            angle_snap: 15.,
            tool: 0,
            orthographic: false,
            remember_camera: true,
            target: c.target.to_array(),
            yaw: c.yaw,
            pitch: c.pitch,
            distance: c.distance,
        }
    }
}
impl Preferences {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1 && self.tool <= 2,
            "Unsupported preferences version/tool"
        );
        ensure!(
            [0., 0.1, 0.5, 1., 5.].contains(&self.move_snap),
            "Invalid movement snap"
        );
        ensure!(
            [0., 5., 15., 45., 90.].contains(&self.angle_snap),
            "Invalid angle snap"
        );
        ensure!(
            self.target.iter().all(|n| n.is_finite() && n.abs() <= 1e7)
                && self.yaw.is_finite()
                && self.yaw.abs() <= 1e7
                && self.pitch.is_finite()
                && self.pitch.abs() <= std::f64::consts::FRAC_PI_2
                && (1. ..=20000.).contains(&self.distance),
            "Invalid camera preferences"
        );
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(16385)
            .read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= 16384, "Preferences exceed 16 KB");
        let p: Self = serde_json::from_slice(&bytes)?;
        p.validate()?;
        Ok(p)
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        storage::atomic_write(path, &serde_json::to_vec_pretty(self)?)
    }
}
