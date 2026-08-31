/**
 * Generates MTT app icon PNG from SVG source, then runs `tauri icon` to produce the full icon set.
 * Requires: Node 18+, @tauri-apps/cli (devDependency), and one of:
 *   - resvg-js (optional devDependency), or
 *   - PowerShell + System.Drawing on Windows (fallback)
 */
import { execSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const iconsDir = join(root, "src-tauri", "icons");
const svgPath = join(iconsDir, "icon.svg");
const sourcePng = join(iconsDir, "icon-source.png");

async function renderWithResvg() {
  const { Resvg } = await import("@resvg/resvg-js");
  const svg = readFileSync(svgPath, "utf8");
  const resvg = new Resvg(svg, { fitTo: { mode: "width", value: 1024 } });
  const pngData = resvg.render().asPng();
  writeFileSync(sourcePng, pngData);
}

function renderWithPowerShell() {
  const ps = `
Add-Type -AssemblyName System.Drawing
$size = 1024
$bmp = New-Object System.Drawing.Bitmap $size, $size
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
$rect = New-Object System.Drawing.Rectangle 0, 0, $size, $size
$brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush $rect, ([System.Drawing.Color]::FromArgb(37, 99, 235)), ([System.Drawing.Color]::FromArgb(20, 184, 166)), 45.0
$path = New-Object System.Drawing.Drawing2D.GraphicsPath
$radius = 192.0
$path.AddArc(0, 0, $radius * 2, $radius * 2, 180, 90)
$path.AddArc($size - $radius * 2, 0, $radius * 2, $radius * 2, 270, 90)
$path.AddArc($size - $radius * 2, $size - $radius * 2, $radius * 2, $radius * 2, 0, 90)
$path.AddArc(0, $size - $radius * 2, $radius * 2, $radius * 2, 90, 90)
$path.CloseFigure()
$g.FillPath($brush, $path)
$fontFamily = New-Object System.Drawing.FontFamily "Segoe UI"
$font = New-Object System.Drawing.Font $fontFamily, 320, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel
$textBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
$sf = New-Object System.Drawing.StringFormat
$sf.Alignment = [System.Drawing.StringAlignment]::Center
$sf.LineAlignment = [System.Drawing.StringAlignment]::Center
$g.DrawString("MTT", $font, $textBrush, [System.Drawing.RectangleF]::new(0, 0, $size, $size), $sf)
$out = "${sourcePng.replace(/\\/g, "\\\\")}"
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
`;
  execSync(`powershell -NoProfile -Command "${ps.replace(/"/g, '\\"').replace(/\r?\n/g, "; ")}"`, {
    stdio: "inherit",
  });
}

mkdirSync(iconsDir, { recursive: true });

if (!existsSync(svgPath)) {
  console.error("Missing icon.svg at", svgPath);
  process.exit(1);
}

try {
  await renderWithResvg();
  console.log("Rendered icon-source.png with @resvg/resvg-js");
} catch {
  console.log("resvg not available, using PowerShell System.Drawing fallback");
  renderWithPowerShell();
}

console.log("Running tauri icon...");
execSync(`npx tauri icon "${sourcePng}" -o "${iconsDir}"`, {
  cwd: root,
  stdio: "inherit",
});

console.log("Icon set generated in", iconsDir);
