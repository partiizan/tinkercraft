use anyhow::{Context, Result, bail};
use std::{io::Write, path::PathBuf};
use tinkercraft_core::{
    Project, evaluate, geometry,
    operations::Selection,
    storage,
    viewport::{self, Camera},
};
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--version") {
        println!("TinkerCraft CLI {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.get(1).is_some_and(|a| a == "self-test") {
        let p = Project::demo();
        let bytes = p.bytes()?;
        let restored = Project::parse(&bytes)?;
        anyhow::ensure!(p == restored, "Project roundtrip failed");
        let mesh = evaluate(&p)?;
        let stl = mesh.stl()?;
        let node = tinkercraft_core::stl::import(&stl, "Self-test", 1.)?;
        let imported = Project {
            parts: vec![node],
            ..Project::default()
        };
        let volume = evaluate(&imported)?.volume();
        anyhow::ensure!(
            (volume / mesh.volume() - 1.).abs() < 1e-6,
            "STL roundtrip volume mismatch"
        );
        println!(
            "{}",
            serde_json::json!({"self_test":"passed","version":env!("CARGO_PKG_VERSION"),"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"schema":p.version,"triangles":mesh.triangles.len(),"volume_mm3":volume})
        );
        return Ok(());
    }
    let Some(command) = args.get(1) else {
        println!(
            "TinkerCraft CLI\n  --version\n  self-test\n  import INPUT.stl OUTPUT.tinkercraft [mm|in|cm]\n  demo OUTPUT.tinkercraft\n  validate INPUT.tinkercraft\n  export INPUT.tinkercraft OUTPUT.stl\n  export-3mf INPUT.tinkercraft OUTPUT.3mf\n  render INPUT.tinkercraft OUTPUT.ppm"
        );
        return Ok(());
    };
    let input = args.get(2).context("Missing project path")?;
    if command == "demo" {
        storage::atomic_write(&PathBuf::from(input), &Project::demo().bytes()?)?;
        return Ok(());
    }
    if command == "import" {
        let output = args.get(3).context("Missing output project path")?;
        let scale = match args.get(4).map(String::as_str).unwrap_or("mm") {
            "mm" => 1.,
            "in" => 25.4,
            "cm" => 10.,
            _ => bail!("Units must be mm, in, or cm"),
        };
        let path = PathBuf::from(input);
        let node = tinkercraft_core::stl::import(
            &tinkercraft_core::stl::read(&path)?,
            &path.file_stem().unwrap_or_default().to_string_lossy(),
            scale,
        )?;
        let p = Project {
            parts: vec![node],
            ..Project::default()
        };
        storage::atomic_write(&PathBuf::from(output), &p.bytes()?)?;
        return Ok(());
    }
    let p = Project::parse(&storage::read(&PathBuf::from(input))?)?;
    match command.as_str() {
        "validate" => {
            let m = evaluate(&p)?;
            println!(
                "{}",
                serde_json::json!({"name":p.name,"schema":p.version,"objects":p.parts.len(),"triangles":m.triangles.len(),"volume_mm3":m.volume()})
            );
        }
        "export-3mf" => {
            let out = args.get(3).context("Missing output path")?;
            storage::atomic_write(
                &PathBuf::from(out),
                &tinkercraft_core::three_mf::export(&p)?,
            )?;
        }
        "export" => {
            let out = args.get(3).context("Missing output path")?;
            storage::atomic_write(&PathBuf::from(out), &evaluate(&p)?.stl()?)?;
        }
        "render" => {
            let out = args.get(3).context("Missing output path")?;
            let m = evaluate(&p)?;
            let mut camera = Camera::default();
            let mut bounds = geometry::Bounds::default();
            for n in &p.parts {
                bounds.merge(geometry::bounds(n));
            }
            camera.fit(bounds, 4. / 3.);
            let frame = viewport::render(&camera, 960, 720, &[], &Selection::new(), Some(&m));
            let mut file = std::fs::File::create(out)?;
            write!(file, "P6\n960 720\n255\n")?;
            for pixel in frame.rgba.chunks(4) {
                file.write_all(&pixel[..3])?;
            }
        }
        _ => bail!("Unknown command"),
    };
    Ok(())
}
