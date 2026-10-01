use anyhow::{Context, Result};
use music_engraver::render::png::svg_to_png;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Svg,
    Png,
    Pdf,
}

impl OutputFormat {
    pub fn resolve(explicit: Option<&str>, output: Option<&str>, default: Self) -> Result<Self> {
        let raw = explicit.or_else(|| {
            output.and_then(|path| Path::new(path).extension().and_then(|ext| ext.to_str()))
        });
        match raw.map(str::to_ascii_lowercase).as_deref() {
            None => Ok(default),
            Some("text" | "txt") => Ok(Self::Text),
            Some("json") => Ok(Self::Json),
            Some("svg") => Ok(Self::Svg),
            Some("png") => Ok(Self::Png),
            Some("pdf") => Ok(Self::Pdf),
            Some(other) => anyhow::bail!(
                "unsupported output format '{other}' (options: text, json, svg, png, pdf)"
            ),
        }
    }

    pub fn is_binary(self) -> bool {
        matches!(self, Self::Png | Self::Pdf)
    }
}

pub fn write_output(format: OutputFormat, output: Option<&str>, bytes: &[u8]) -> Result<()> {
    if format.is_binary() {
        let path = output.context("PNG/PDF output requires --output <path>")?;
        fs::write(path, bytes).with_context(|| format!("failed to write {path}"))?;
    } else if let Some(path) = output {
        fs::write(path, bytes).with_context(|| format!("failed to write {path}"))?;
    } else {
        io::stdout().write_all(bytes)?;
    }
    Ok(())
}

pub fn svg_bytes(svg: String, format: OutputFormat, dpi: f32) -> Result<Vec<u8>> {
    match format {
        OutputFormat::Svg => Ok(svg.into_bytes()),
        OutputFormat::Png => {
            anyhow::ensure!(dpi > 0.0, "--dpi must be positive");
            svg_to_png(&svg, dpi / 96.0).context("failed to rasterize SVG")
        }
        OutputFormat::Pdf => svg_to_pdf(&svg),
        _ => anyhow::bail!("internal error: non-graphical format passed to SVG renderer"),
    }
}

fn svg_to_pdf(svg: &str) -> Result<Vec<u8>> {
    let mut child = Command::new("rsvg-convert")
        .args(["--format", "pdf"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("PDF output requires the 'rsvg-convert' executable")?;
    child
        .stdin
        .take()
        .context("failed to open rsvg-convert stdin")?
        .write_all(svg.as_bytes())
        .context("failed to send SVG to rsvg-convert")?;
    let output = child
        .wait_with_output()
        .context("failed to wait for rsvg-convert")?;
    anyhow::ensure!(
        output.status.success(),
        "rsvg-convert failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(output.stdout)
}
