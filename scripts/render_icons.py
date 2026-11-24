import os
from PIL import Image, ImageDraw

SVG_CONTENT = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="100%" height="100%">
  <defs>
    <linearGradient id="auri-glyph" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" stop-color="#A855F7"/>
      <stop offset="100%" stop-color="#6366F1"/>
    </linearGradient>
  </defs>
  
  <!-- Apple / Spotify Minimalist Squircle Base -->
  <rect x="32" y="32" width="448" height="448" rx="104" ry="104" fill="#0F111A"/>
  
  <!-- Sleek Geometric Apex "A" with Rounded Caps -->
  <path d="M 140 396 L 240 134 Q 256 96 272 134 L 372 396"
        fill="none"
        stroke="url(#auri-glyph)"
        stroke-width="38"
        stroke-linecap="round"
        stroke-linejoin="round"/>
        
  <!-- Minimalist Audio Waveform Floating Sound Bars -->
  <rect x="228" y="218" width="56" height="24" rx="12" fill="url(#auri-glyph)"/>
  <rect x="208" y="272" width="96" height="26" rx="13" fill="url(#auri-glyph)"/>
  <rect x="188" y="326" width="136" height="28" rx="14" fill="url(#auri-glyph)"/>
</svg>
"""

def render_master(target_size=2048):
    scale = target_size / 512.0
    
    # 1. Background squircle
    bg = Image.new('RGBA', (target_size, target_size), (0, 0, 0, 0))
    d_bg = ImageDraw.Draw(bg)
    margin = int(32 * scale)
    radius = int(104 * scale)
    d_bg.rounded_rectangle([margin, margin, target_size - margin, target_size - margin], radius=radius, fill=(15, 17, 26, 255))
    
    # 2. Vector glyph mask
    mask = Image.new('L', (target_size, target_size), 0)
    d_m = ImageDraw.Draw(mask)
    
    stroke_w = int(38 * scale)
    pts = []
    steps = 60
    
    # Left leg: (140, 396) -> (240, 134)
    for i in range(steps + 1):
        t = i / float(steps)
        x = (140.0 + (240.0 - 140.0) * t) * scale
        y = (396.0 + (134.0 - 396.0) * t) * scale
        pts.append((x, y))
        
    # Apex quadratic curve: Q (256, 96) -> (272, 134)
    p0 = (240.0, 134.0)
    p1 = (256.0, 96.0)
    p2 = (272.0, 134.0)
    for i in range(1, steps + 1):
        t = i / float(steps)
        x = ((1.0 - t)**2 * p0[0] + 2.0 * (1.0 - t) * t * p1[0] + t**2 * p2[0]) * scale
        y = ((1.0 - t)**2 * p0[1] + 2.0 * (1.0 - t) * t * p1[1] + t**2 * p2[1]) * scale
        pts.append((x, y))
        
    # Right leg: (272, 134) -> (372, 396)
    for i in range(1, steps + 1):
        t = i / float(steps)
        x = (272.0 + (372.0 - 272.0) * t) * scale
        y = (134.0 + (396.0 - 134.0) * t) * scale
        pts.append((x, y))
        
    d_m.line(pts, fill=255, width=stroke_w, joint='curve')
    hw = stroke_w // 2
    f_left = (int(140.0 * scale), int(396.0 * scale))
    f_right = (int(372.0 * scale), int(396.0 * scale))
    d_m.ellipse([f_left[0] - hw, f_left[1] - hw, f_left[0] + hw, f_left[1] + hw], fill=255)
    d_m.ellipse([f_right[0] - hw, f_right[1] - hw, f_right[0] + hw, f_right[1] + hw], fill=255)
    
    # Bar 1 (top): center y=230, width=56, height=24, rx=12
    y1 = int(230.0 * scale)
    w1 = int(56.0 * scale)
    h1 = int(24.0 * scale)
    cx = int(256.0 * scale)
    d_m.rounded_rectangle([cx - w1 // 2, y1 - h1 // 2, cx + w1 // 2, y1 + h1 // 2], radius=h1 // 2, fill=255)
    
    # Bar 2 (middle): center y=285, width=96, height=26, rx=13
    y2 = int(285.0 * scale)
    w2 = int(96.0 * scale)
    h2 = int(26.0 * scale)
    d_m.rounded_rectangle([cx - w2 // 2, y2 - h2 // 2, cx + w2 // 2, y2 + h2 // 2], radius=h2 // 2, fill=255)
    
    # Bar 3 (bottom): center y=340, width=136, height=28, rx=14
    y3 = int(340.0 * scale)
    w3 = int(136.0 * scale)
    h3 = int(28.0 * scale)
    d_m.rounded_rectangle([cx - w3 // 2, y3 - h3 // 2, cx + w3 // 2, y3 + h3 // 2], radius=h3 // 2, fill=255)
    
    # 3. Smooth vertical gradient for glyph: Electric Violet (#A855F7) -> Electric Indigo (#6366F1)
    c_top = (168, 85, 247)
    c_bot = (99, 102, 241)
    y_min = int(96.0 * scale)
    y_max = int(415.0 * scale)
    dy = max(1.0, float(y_max - y_min))
    
    grad = Image.new('RGBA', (target_size, target_size), (0, 0, 0, 0))
    for y in range(target_size):
        t = max(0.0, min(1.0, (y - y_min) / dy))
        r = int(c_top[0] * (1.0 - t) + c_bot[0] * t)
        g = int(c_top[1] * (1.0 - t) + c_bot[1] * t)
        b = int(c_top[2] * (1.0 - t) + c_bot[2] * t)
        for x in range(target_size):
            grad.putpixel((x, y), (r, g, b, 255))
    grad.putalpha(mask)
    
    return Image.alpha_composite(bg, grad)

def main():
    os.makedirs('assets', exist_ok=True)
    
    # 1. Write clean SVG
    svg_path = 'assets/icon.svg'
    with open(svg_path, 'w', encoding='utf-8') as f:
        f.write(SVG_CONTENT.strip() + '\n')
    print(f"Wrote {svg_path} ({len(SVG_CONTENT)} bytes)")
    
    # 2. Render master 2048x2048 image with antialiased supersampling
    print("Rendering 2048x2048 master image...")
    master = render_master(2048)
    
    # 3. Generate assets/icon.png (512x512)
    icon_512 = master.resize((512, 512), Image.Resampling.LANCZOS)
    icon_512.save('assets/icon.png', format='PNG', optimize=True)
    print(f"Saved assets/icon.png (512x512, {os.path.getsize('assets/icon.png')} bytes)")
    
    # 4. Generate assets/brand_64.png (64x64) for in-app ui::app_icon()
    brand_64 = master.resize((64, 64), Image.Resampling.LANCZOS)
    brand_64.save('assets/brand_64.png', format='PNG', optimize=True)
    print(f"Saved assets/brand_64.png (64x64, {os.path.getsize('assets/brand_64.png')} bytes)")
    
    # 5. Generate assets/icon.ico with all standard Windows resolutions
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    ico_path = 'assets/icon.ico'
    icon_512.save(ico_path, format='ICO', sizes=ico_sizes)
    print(f"Saved {ico_path} (sizes={ico_sizes}, {os.path.getsize(ico_path)} bytes)")

if __name__ == '__main__':
    main()
