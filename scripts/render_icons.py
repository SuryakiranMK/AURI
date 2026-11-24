"""
AURI Icon Asset Generator
Generates multi-resolution assets/icon.ico and assets/brand_64.png from assets/icon.png.
"""
import os
from PIL import Image

def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    assets_dir = os.path.join(root, "assets")
    icon_png = os.path.join(assets_dir, "icon.png")
    
    if not os.path.exists(icon_png):
        raise FileNotFoundError(f"Source icon not found at {icon_png}")
        
    img = Image.open(icon_png)
    print(f"Loaded master icon: {img.size}, mode: {img.mode}")
    
    # 1. Generate assets/brand_64.png (64x64)
    brand_64_path = os.path.join(assets_dir, "brand_64.png")
    brand_64 = img.resize((64, 64), Image.Resampling.LANCZOS)
    brand_64.save(brand_64_path, format="PNG", optimize=True)
    print(f"Generated {brand_64_path} ({os.path.getsize(brand_64_path)} bytes)")
    
    # 2. Generate assets/icon.ico with multi-resolutions (16, 24, 32, 48, 64, 128, 256)
    ico_path = os.path.join(assets_dir, "icon.ico")
    ico_sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    img.save(ico_path, format="ICO", sizes=ico_sizes)
    print(f"Generated {ico_path} ({os.path.getsize(ico_path)} bytes)")

if __name__ == "__main__":
    main()
