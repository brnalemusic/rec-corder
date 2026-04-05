$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Drawing

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = (Resolve-Path (Join-Path $scriptDir '..')).Path
$outputDir = Join-Path $rootDir 'src-tauri\installer-assets'
$logoPath = Join-Path $rootDir 'src\assets\logo.png'
$versionPath = Join-Path $rootDir 'version.txt'

if (-not (Test-Path $logoPath)) {
  throw "Logo not found at $logoPath"
}

if (-not (Test-Path $versionPath)) {
  throw "Version file not found at $versionPath"
}

$version = (Get-Content $versionPath -Raw).Trim()
New-Item -ItemType Directory -Path $outputDir -Force | Out-Null

function New-Color($hex) {
  return [System.Drawing.ColorTranslator]::FromHtml($hex)
}

function New-Font($family, $size, $style = [System.Drawing.FontStyle]::Regular) {
  return New-Object System.Drawing.Font($family, $size, $style, [System.Drawing.GraphicsUnit]::Pixel)
}

function Set-GraphicsQuality($graphics) {
  $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit
}

function Save-Bitmap($bitmap, $path) {
  $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Bmp)
  $bitmap.Dispose()
}

function New-AccentBrush() {
  return New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    (New-Object System.Drawing.Rectangle 0, 0, 1, 320),
    (New-Color '#FF7F39'),
    (New-Color '#E87530'),
    90
  )
}

function Draw-BrandGlow($graphics, $x, $y, $width, $height, $color) {
  $brush = New-Object System.Drawing.SolidBrush $color
  $graphics.FillEllipse($brush, $x, $y, $width, $height)
  $brush.Dispose()
}

function Draw-RoundedPanel($graphics, $rectangle, $fillColor, $borderColor) {
  $path = New-Object System.Drawing.Drawing2D.GraphicsPath
  $radius = 16
  $diameter = $radius * 2

  $path.AddArc($rectangle.X, $rectangle.Y, $diameter, $diameter, 180, 90)
  $path.AddArc($rectangle.Right - $diameter, $rectangle.Y, $diameter, $diameter, 270, 90)
  $path.AddArc($rectangle.Right - $diameter, $rectangle.Bottom - $diameter, $diameter, $diameter, 0, 90)
  $path.AddArc($rectangle.X, $rectangle.Bottom - $diameter, $diameter, $diameter, 90, 90)
  $path.CloseFigure()

  $fillBrush = New-Object System.Drawing.SolidBrush $fillColor
  $borderPen = New-Object System.Drawing.Pen $borderColor, 1

  $graphics.FillPath($fillBrush, $path)
  $graphics.DrawPath($borderPen, $path)

  $fillBrush.Dispose()
  $borderPen.Dispose()
  $path.Dispose()
}

$sidebar = New-Object System.Drawing.Bitmap 164, 314
$sidebarGraphics = [System.Drawing.Graphics]::FromImage($sidebar)
Set-GraphicsQuality $sidebarGraphics

$sidebarBackground = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
  (New-Object System.Drawing.Rectangle 0, 0, 164, 314),
  (New-Color '#0F1117'),
  (New-Color '#161B22'),
  90
)
$sidebarGraphics.FillRectangle($sidebarBackground, 0, 0, 164, 314)
$sidebarBackground.Dispose()

Draw-BrandGlow $sidebarGraphics -48 190 170 170 (New-Color '#1D2530')
Draw-BrandGlow $sidebarGraphics 80 -40 120 120 (New-Color '#FF7F39')

$accentBrush = New-AccentBrush
$sidebarGraphics.FillRectangle($accentBrush, 0, 0, 164, 10)
$accentBrush.Dispose()

$logoImage = [System.Drawing.Image]::FromFile($logoPath)
$logoSize = 74
$logoX = [int]((164 - $logoSize) / 2)
$sidebarGraphics.DrawImage($logoImage, $logoX, 28, $logoSize, $logoSize)

$titleBrush = New-Object System.Drawing.SolidBrush (New-Color '#F0F6FC')
$mutedBrush = New-Object System.Drawing.SolidBrush (New-Color '#8B949E')
$accentTextBrush = New-Object System.Drawing.SolidBrush (New-Color '#FFB38A')

$titleFont = New-Font 'Segoe UI Semibold' 21
$subtitleFont = New-Font 'Segoe UI' 10
$metaFont = New-Font 'Segoe UI Semibold' 9
$featureFont = New-Font 'Segoe UI' 9

$titleFormat = New-Object System.Drawing.StringFormat
$titleFormat.Alignment = [System.Drawing.StringAlignment]::Center
$titleFormat.LineAlignment = [System.Drawing.StringAlignment]::Near

$sidebarGraphics.DrawString('Rec Corder', $titleFont, $titleBrush, (New-Object System.Drawing.RectangleF 10, 114, 144, 34), $titleFormat)
$sidebarGraphics.DrawString('Ultra-light screen recorder', $subtitleFont, $mutedBrush, (New-Object System.Drawing.RectangleF 18, 148, 128, 28), $titleFormat)
$sidebarGraphics.DrawString("v$version", $metaFont, $accentTextBrush, (New-Object System.Drawing.RectangleF 18, 176, 128, 18), $titleFormat)

$panelRect = New-Object System.Drawing.Rectangle 14, 214, 136, 74
Draw-RoundedPanel $sidebarGraphics $panelRect (New-Color '#1C2128') (New-Color '#30363D')
$sidebarGraphics.DrawString('GPU-first capture', $metaFont, $titleBrush, (New-Object System.Drawing.PointF 26, 230))
$sidebarGraphics.DrawString('FFmpeg bundled', $featureFont, $mutedBrush, (New-Object System.Drawing.PointF 26, 248))
$sidebarGraphics.DrawString('Fast recovery flow', $featureFont, $mutedBrush, (New-Object System.Drawing.PointF 26, 264))

$titleFormat.Dispose()
$titleFont.Dispose()
$subtitleFont.Dispose()
$metaFont.Dispose()
$featureFont.Dispose()
$titleBrush.Dispose()
$mutedBrush.Dispose()
$accentTextBrush.Dispose()
$logoImage.Dispose()
$sidebarGraphics.Dispose()

Save-Bitmap $sidebar (Join-Path $outputDir 'sidebar.bmp')

$header = New-Object System.Drawing.Bitmap 150, 57
$headerGraphics = [System.Drawing.Graphics]::FromImage($header)
Set-GraphicsQuality $headerGraphics

$headerBackground = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
  (New-Object System.Drawing.Rectangle 0, 0, 150, 57),
  (New-Color '#FF7F39'),
  (New-Color '#E87530'),
  0
)
$headerGraphics.FillRectangle($headerBackground, 0, 0, 150, 57)
$headerBackground.Dispose()

$overlayBrush = New-Object System.Drawing.SolidBrush (New-Color '#FFB38A')
$headerGraphics.FillEllipse($overlayBrush, 88, -24, 92, 92)
$overlayBrush.Dispose()

$headerLogo = [System.Drawing.Image]::FromFile($logoPath)
$headerGraphics.DrawImage($headerLogo, 10, 8, 40, 40)

$headerTitleFont = New-Font 'Segoe UI Semibold' 16
$headerSubtitleFont = New-Font 'Segoe UI' 8
$headerTitleBrush = New-Object System.Drawing.SolidBrush (New-Color '#FFFFFF')
$headerSubtitleBrush = New-Object System.Drawing.SolidBrush (New-Color '#FFF0E8')

$headerGraphics.DrawString('Rec Corder', $headerTitleFont, $headerTitleBrush, (New-Object System.Drawing.PointF 56, 10))
$headerGraphics.DrawString('Install for zero-lag capture', $headerSubtitleFont, $headerSubtitleBrush, (New-Object System.Drawing.PointF 57, 31))

$headerLogo.Dispose()
$headerTitleFont.Dispose()
$headerSubtitleFont.Dispose()
$headerTitleBrush.Dispose()
$headerSubtitleBrush.Dispose()
$headerGraphics.Dispose()

Save-Bitmap $header (Join-Path $outputDir 'header.bmp')

Write-Host "Installer assets regenerated in $outputDir"
