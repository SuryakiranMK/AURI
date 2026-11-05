# Pure Python script to generate sleek AURI placeholder app icon (PNG & multi-res ICO)
# Zero external dependencies (uses standard library struct & zlib).

import struct
import zlib
import math
import os

def create_png(width, height, draw_fn):
    # RGBA image buffer: list of rows, each row has width * 4 bytes
    rows = []
    for y in range(height):
        row = bytearray()
        for x in range(width):
            r, g, b, a = draw_fn(x, y, width, height)
            row.extend([r, g, b, a])
        rows.append(b'\x00' + bytes(row))  # Filter byte 0 (None)
    
    raw_data = b''.join(rows)
    compressed = zlib.compress(raw_data, level=9)
    
    def chunk(tag, data):
        length = struct.pack('>I', len(data))
        crc = struct.pack('>I', zlib.crc32(tag + data) & 0xffffffff)
        return length + tag + data + crc

    png_header = b'\x89PNG\r\n\x1a\n'
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr = chunk(b'IHDR', ihdr_data)
    idat = chunk(b'IDAT', compressed)
    iend = chunk(b'IEND', b'')
    
    return png_header + ihdr + idat + iend

def auri_pixel(x, y, w, h):
    # Normalized coords (-1.0 to 1.0)
    nx = (x / (w - 1)) * 2.0 - 1.0
    ny = (y / (h - 1)) * 2.0 - 1.0
    dist = math.sqrt(nx * nx + ny * ny)

    # Squircle shape boundary: (x^4 + y^4) < 0.75^4
    squircle_dist = (nx**4 + ny**4)**0.25
    if squircle_dist > 0.92:
        # Transparent outside
        return (0, 0, 0, 0)

    # Background gradient: Dark obsidian #0E0F14 to #181924
    bg_t = (ny + 1.0) * 0.5
    bg_r = int(14 * (1.0 - bg_t) + 24 * bg_t)
    bg_g = int(15 * (1.0 - bg_t) + 25 * bg_t)
    bg_b = int(20 * (1.0 - bg_t) + 36 * bg_t)
    bg_a = 255

    # Subtle squircle border highlight
    if squircle_dist > 0.88:
        edge_glow = (squircle_dist - 0.88) / 0.04
        bg_r = int(bg_r * (1.0 - edge_glow) + 99 * edge_glow)
        bg_g = int(bg_g * (1.0 - edge_glow) + 102 * edge_glow)
        bg_b = int(bg_b * (1.0 - edge_glow) + 241 * edge_glow)

    # Stylized AURI "A" Soundwave Monogram:
    # Outer A leg: Left diagonal (from nx=-0.48, ny=0.55 to nx=0.0, ny=-0.58)
    # Right diagonal (from nx=0.48, ny=0.55 to nx=0.0, ny=-0.58)
    # Center soundwave bars across the crossbar
    
    # Distance to left diagonal line segment
    def dist_to_segment(px, py, x1, y1, x2, y2):
        dx, dy = x2 - x1, y2 - y1
        l2 = dx*dx + dy*dy
        if l2 == 0: return math.hypot(px-x1, py-y1)
        t = max(0.0, min(1.0, ((px - x1) * dx + (py - y1) * dy) / l2))
        proj_x = x1 + t * dx
        proj_y = y1 + t * dy
        return math.hypot(px - proj_x, py - proj_y)

    d_left = dist_to_segment(nx, ny, -0.42, 0.48, 0.0, -0.52)
    d_right = dist_to_segment(nx, ny, 0.42, 0.48, 0.0, -0.52)
    d_leg = min(d_left, d_right)

    # Crossbars / audio waveform inside A:
    # Waveform bar 1 (center): x in [-0.04, 0.04], y in [-0.05, 0.28]
    # Waveform bar 2 (left):   x in [-0.18, -0.11], y in [0.08, 0.28]
    # Waveform bar 3 (right):  x in [0.11, 0.18], y in [0.08, 0.28]
    
    in_bar1 = (abs(nx) <= 0.045) and (-0.12 <= ny <= 0.28)
    in_bar2 = (-0.20 <= nx <= -0.11) and (0.02 <= ny <= 0.28)
    in_bar3 = (0.11 <= nx <= 0.20) and (0.02 <= ny <= 0.28)

    # Pulse concentric rings (audio soundwaves)
    arc_ring = False
    ring_r = math.hypot(nx, ny + 0.52)
    if 0.18 <= ring_r <= 0.24 and ny < -0.3:
        arc_ring = True

    # Render foreground elements
    if d_leg < 0.075 or in_bar1 or in_bar2 or in_bar3 or arc_ring:
        # Dynamic indigo-to-purple gradient: #6366F1 -> #A855F7
        t_grad = (ny + 0.6) / 1.1
        t_grad = max(0.0, min(1.0, t_grad))
        fg_r = int(99 * (1.0 - t_grad) + 168 * t_grad)
        fg_g = int(102 * (1.0 - t_grad) + 85 * t_grad)
        fg_b = int(241 * (1.0 - t_grad) + 247 * t_grad)
        
        # Soft antialiasing for legs
        if d_leg < 0.075:
            alpha = min(1.0, (0.075 - d_leg) / 0.018)
            r = int(bg_r * (1.0 - alpha) + fg_r * alpha)
            g = int(bg_g * (1.0 - alpha) + fg_g * alpha)
            b = int(bg_b * (1.0 - alpha) + fg_b * alpha)
            return (r, g, b, 255)
        return (fg_r, fg_g, fg_b, 255)

    return (bg_r, bg_g, bg_b, bg_a)

def generate_assets():
    os.makedirs('assets', exist_ok=True)
    
    sizes = [256, 128, 64, 48, 32, 16]
    png_buffers = {}
    
    print("Generating AURI placeholder icon assets...")
    for s in sizes:
        png_data = create_png(s, s, auri_pixel)
        png_buffers[s] = png_data
        if s == 256:
            with open('assets/icon.png', 'wb') as f:
                f.write(png_data)
            print(f"  [+] assets/icon.png ({s}x{s}, {len(png_data)} bytes)")

    # Build Windows Multi-Resolution ICO containing all PNG representations
    ico_header = struct.pack('<HHH', 0, 1, len(sizes))
    entries = []
    offset = 6 + 16 * len(sizes)
    
    for s in sizes:
        png_bytes = png_buffers[s]
        width_byte = 0 if s == 256 else s
        height_byte = 0 if s == 256 else s
        entry = struct.pack('<BBBBHHII',
                            width_byte, height_byte,
                            0, 0,
                            1, 32,
                            len(png_bytes), offset)
        entries.append(entry)
        offset += len(png_bytes)

    with open('assets/icon.ico', 'wb') as f:
        f.write(ico_header)
        for e in entries:
            f.write(e)
        for s in sizes:
            f.write(png_buffers[s])
            
    print(f"  [+] assets/icon.ico ({len(sizes)} resolutions: 256, 128, 64, 48, 32, 16, size: {os.path.getsize('assets/icon.ico')} bytes)")

if __name__ == '__main__':
    generate_assets()

