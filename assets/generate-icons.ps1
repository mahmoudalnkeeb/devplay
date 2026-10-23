# Regenerates the icons: powershell -ExecutionPolicy Bypass -File assets\generate-icons.ps1

Add-Type -AssemblyName System.Drawing

$Slate = [System.Drawing.ColorTranslator]::FromHtml('#0F172A')
$Amber = [System.Drawing.ColorTranslator]::FromHtml('#F59E0B')
$Sky   = [System.Drawing.ColorTranslator]::FromHtml('#38BDF8')
$White = [System.Drawing.Color]::White

$AppSizes  = 16, 20, 24, 32, 40, 48, 64, 256
$TraySizes = 16, 20, 24, 32, 40, 48

function New-RoundedRect([float]$x, [float]$y, [float]$w, [float]$h, [float]$r) {
    $p = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = 2 * $r
    $p.AddArc($x, $y, $d, $d, 180, 90)
    $p.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
    $p.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90)
    $p.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
    $p.CloseFigure()
    $p
}

# Coordinates are in a 0..1 square, scaled to the target size by Render.
function Draw-Icon($g, $triangleColor) {
    $g.FillPath((New-Object System.Drawing.SolidBrush $Slate), (New-RoundedRect 0.03 0.03 0.94 0.94 0.22))

    $triangle = [System.Drawing.PointF[]]@(
        (New-Object System.Drawing.PointF 0.27, 0.27),
        (New-Object System.Drawing.PointF 0.73, 0.50),
        (New-Object System.Drawing.PointF 0.27, 0.73))
    $g.FillPolygon((New-Object System.Drawing.SolidBrush $triangleColor), $triangle)
    $rounding = New-Object System.Drawing.Pen $triangleColor, 0.06
    $rounding.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    $g.DrawPolygon($rounding, $triangle)

    $g.FillRectangle((New-Object System.Drawing.SolidBrush $White), [float]0.66, [float]0.63, [float]0.17, [float]0.075)
}

function Render([System.Drawing.Color]$triangleColor, [int]$size) {
    $bmp = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.ScaleTransform($size, $size)
    Draw-Icon $g $triangleColor
    $g.Dispose()
    $bmp
}

# PNG for 256 px; 32-bit DIB below that, which CreateIconFromResourceEx loads reliably.
function Get-ImageBytes($bmp) {
    $size = $bmp.Width
    $ms = New-Object System.IO.MemoryStream
    if ($size -ge 256) {
        $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
        return , $ms.ToArray()
    }
    $w = New-Object System.IO.BinaryWriter $ms
    $maskStride = [int]([math]::Ceiling($size / 32.0) * 4)
    $w.Write([int32]40); $w.Write([int32]$size); $w.Write([int32]($size * 2))
    $w.Write([int16]1); $w.Write([int16]32); $w.Write([int32]0)
    $w.Write([int32]($size * $size * 4 + $maskStride * $size))
    $w.Write([int32]0); $w.Write([int32]0); $w.Write([int32]0); $w.Write([int32]0)
    $rect = New-Object System.Drawing.Rectangle 0, 0, $size, $size
    $data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $row = New-Object byte[] ($size * 4)
    for ($y = $size - 1; $y -ge 0; $y--) {
        [System.Runtime.InteropServices.Marshal]::Copy([IntPtr]($data.Scan0.ToInt64() + $y * $data.Stride), $row, 0, $row.Length)
        $w.Write($row)
    }
    $bmp.UnlockBits($data)
    $w.Write((New-Object byte[] ($maskStride * $size)))
    $w.Flush()
    , $ms.ToArray()
}

function Write-Ico([System.Drawing.Color]$triangleColor, [int[]]$sizes, [string]$path) {
    $images = foreach ($s in $sizes) { , (Get-ImageBytes (Render $triangleColor $s)) }
    $w = New-Object System.IO.BinaryWriter ([System.IO.File]::Create($path))
    $w.Write([int16]0); $w.Write([int16]1); $w.Write([int16]$sizes.Count)
    $offset = 6 + 16 * $sizes.Count
    for ($i = 0; $i -lt $sizes.Count; $i++) {
        $dim = if ($sizes[$i] -ge 256) { 0 } else { $sizes[$i] }
        $w.Write([byte]$dim); $w.Write([byte]$dim); $w.Write([byte]0); $w.Write([byte]0)
        $w.Write([int16]1); $w.Write([int16]32)
        $w.Write([int32]$images[$i].Length); $w.Write([int32]$offset)
        $offset += $images[$i].Length
    }
    foreach ($img in $images) { $w.Write($img) }
    $w.Close()
}

Write-Ico $Amber $AppSizes  (Join-Path $PSScriptRoot 'devplay.ico')
Write-Ico $Amber $TraySizes (Join-Path $PSScriptRoot 'tray-play.ico')
Write-Ico $Sky   $TraySizes (Join-Path $PSScriptRoot 'tray-dev.ico')
(Render $Amber 256).Save((Join-Path $PSScriptRoot 'devplay.png'), [System.Drawing.Imaging.ImageFormat]::Png)
