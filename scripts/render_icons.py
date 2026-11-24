import os
from PIL import Image, ImageDraw

def render_master(target_size=1024):
    scale = target_size / 512.0
    img = Image.new('RGBA', (target_size, target_size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # 1. Background squircle
    margin = int(32 * scale)
    radius = int(104 * scale)
    border_w = max(1, int(3 * scale))
    draw.rounded_rectangle(
        [margin, margin, target_size - margin, target_size - margin],
        radius=radius,
        fill=(20, 23, 36, 255),
        outline=(75, 85, 104, 180),
        width=border_w
    )
    
    # 2. Ambient radial glow
    glow = Image.new('RGBA', (target_size, target_size), (0, 0, 0, 0))
    g_draw = ImageDraw.Draw(glow)
    cx, cy = target_size // 2, int(268 * scale)
    gr = int(180 * scale)
    for r in range(gr, 0, -8):
        alpha = int(36 * (1.0 - r / gr)**1.4)
        g_draw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(139, 92, 246, alpha))
    img = Image.alpha_composite(img, glow)
    
    # 3. Mask for vector glyph
    mask = Image.new('L', (target_size, target_size), 0)
    m_draw = ImageDraw.Draw(mask)
    
    p_left = (int(130 * scale), int(396 * scale))
    p_apex = (int(256 * scale), int(106 * scale))
    p_right = (int(382 * scale), int(396 * scale))
    leg_w = int(42 * scale)
    
    m_draw.line([p_left, p_apex, p_right], fill=255, width=leg_w, joint='curve')
    cb_y = int(278 * scale)
    m_draw.line([(int(170 * scale), cb_y), (int(342 * scale), cb_y)], fill=255, width=int(36 * scale))
    
    c_w = int(30 * scale)
    c_hr = c_w // 2
    c_x = int(256 * scale)
    m_draw.rounded_rectangle(
        [c_x - c_hr, int(154 * scale), c_x + c_hr, int(320 * scale)],
        radius=c_hr, fill=255
    )
    
    l_w = int(28 * scale)
    l_hr = l_w // 2
    l_x = int(208 * scale)
    m_draw.rounded_rectangle(
        [l_x - l_hr, int(212 * scale), l_x + l_hr, int(300 * scale)],
        radius=l_hr, fill=255
    )
    
    r_w = int(28 * scale)
    r_hr = r_w // 2
    r_x = int(304 * scale)
    m_draw.rounded_rectangle(
        [r_x - r_hr, int(212 * scale), r_x + r_hr, int(300 * scale)],
        radius=r_hr, fill=255
    )
    
    # 4. Multi-stop Linear Gradient: Electric Violet -> Indigo -> Cyan -> Emerald
    grad = Image.new('RGBA', (target_size, target_size))
    for y in range(target_size):
        for x in range(target_size):
            t = (x * 0.35 + y * 0.65) / target_size
            t = max(0.0, min(1.0, t))
            if t < 0.38:
                f = t / 0.38
                r = int(168 * (1 - f) + 99 * f)
                g = int(85 * (1 - f) + 102 * f)
                b = int(247 * (1 - f) + 241 * f)
            elif t < 0.72:
                f = (t - 0.38) / 0.34
                r = int(99 * (1 - f) + 6 * f)
                g = int(102 * (1 - f) + 182 * f)
                b = int(241 * (1 - f) + 212 * f)
            else:
                f = (t - 0.72) / 0.28
                r = int(6 * (1 - f) + 16 * f)
                g = int(182 * (1 - f) + 185 * f)
                b = int(212 * (1 - f) + 129 * f)
            grad.putpixel((x, y), (r, g, b, 255))
            
    img.paste(grad, (0, 0), mask)
    return img

if __name__ == '__main__':
    print('Rendering 1024x1024 master image...')
    master = render_master(1024)

    # Export icon.png (512x512)
    png_512 = master.resize((512, 512), Image.Resampling.LANCZOS)
    png_512.save('assets/icon.png', optimize=True)
    print('Saved assets/icon.png (512x512)')

    # Export brand_64.png (64x64)
    png_64 = master.resize((64, 64), Image.Resampling.LANCZOS)
    png_64.save('assets/brand_64.png', optimize=True)
    print('Saved assets/brand_64.png (64x64)')

    # Export icon.ico containing [16, 24, 32, 48, 64, 128, 256]
    master_256 = master.resize((256, 256), Image.Resampling.LANCZOS)
    master_256.save('assets/icon.ico', format='ICO', sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    print('Saved assets/icon.ico with 7 resolutions!')

    # Clean up temp concept files
    for f in os.listdir('.'):
        if f.startswith('concept') or f.startswith('test_final'):
            try:
                os.remove(f)
            except:
                pass
    print('Done!')
