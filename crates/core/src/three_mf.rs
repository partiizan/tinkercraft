//! 3MF Core package: millimeter meshes, base materials and one assembly build item.
use crate::{Project, geometry};
use anyhow::{Result, ensure};
use std::{
    fmt::Write as _,
    io::{Cursor, Write},
};
use zip::{ZipWriter, write::SimpleFileOptions};

fn xml(s: &str) -> String {
    s.chars()
        .filter(|c| *c >= ' ' || matches!(c, '\n' | '\r' | '\t'))
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
pub fn export(p: &Project) -> Result<Vec<u8>> {
    let parts = geometry::colored_parts(p)?;
    ensure!(
        parts.iter().map(|p| p.mesh.triangles.len()).sum::<usize>() <= 3_000_000,
        "3MF export exceeds 3 million triangles"
    );
    let mut colors = Vec::new();
    for part in &parts {
        if !colors.contains(&part.color) {
            colors.push(part.color);
        }
    }
    let mut model = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xml:lang=\"en-US\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n",
    );
    writeln!(
        model,
        "<metadata name=\"Title\">{}</metadata><resources><basematerials id=\"1\">",
        xml(&p.name)
    )?;
    for [r, g, b] in &colors {
        writeln!(
            model,
            "<base name=\"Color #{r:02X}{g:02X}{b:02X}\" displaycolor=\"#{r:02X}{g:02X}{b:02X}FF\"/>"
        )?;
    }
    model.push_str("</basematerials>\n");
    for (i, part) in parts.iter().enumerate() {
        let index = colors.iter().position(|c| *c == part.color).unwrap();
        writeln!(
            model,
            "<object id=\"{}\" type=\"model\" name=\"Part {}\" pid=\"1\" pindex=\"{index}\"><mesh><vertices>",
            i + 2,
            i + 1
        )?;
        for [x, y, z] in &part.mesh.vertices {
            ensure!(
                x.is_finite() && y.is_finite() && z.is_finite(),
                "Non-finite export coordinate"
            );
            writeln!(model, "<vertex x=\"{x}\" y=\"{y}\" z=\"{z}\"/>")?;
        }
        model.push_str("</vertices><triangles>\n");
        for [a, b, c] in &part.mesh.triangles {
            writeln!(model, "<triangle v1=\"{a}\" v2=\"{b}\" v3=\"{c}\"/>")?;
        }
        model.push_str("</triangles></mesh></object>\n");
    }
    let assembly = parts.len() + 2;
    writeln!(
        model,
        "<object id=\"{assembly}\" type=\"model\" name=\"{}\"><components>",
        xml(&p.name)
    )?;
    for i in 0..parts.len() {
        writeln!(model, "<component objectid=\"{}\"/>", i + 2)?;
    }
    writeln!(
        model,
        "</components></object></resources><build><item objectid=\"{assembly}\"/></build></model>"
    )?;
    let content_types = r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/></Types>"#;
    let relationships = r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" Target="/3D/3dmodel.model"/></Relationships>"#;
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (path, data) in [
        ("[Content_Types].xml", content_types),
        ("_rels/.rels", relationships),
        ("3D/3dmodel.model", &model),
    ] {
        zip.start_file(
            path,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
        )?;
        zip.write_all(data.as_bytes())?;
    }
    Ok(zip.finish()?.into_inner())
}
