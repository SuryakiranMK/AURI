use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = r"C:\Users\user\.gemini\antigravity\brain\b931b0bd-d5f9-40c7-acdf-b2da4f69b2d5\.user_uploaded\media_1794113111138_9bac32e7.jpg";
    println!("Loading image from: {}", input_path);

    let img = image::open(input_path)?;
    println!("Original dimensions: {}x{}", img.width(), img.height());

    let assets_dir = Path::new("assets");
    std::fs::create_dir_all(assets_dir)?;

    // 1. Save pristine PNG
    let png_path = assets_dir.join("icon.png");
    img.save_with_format(&png_path, image::ImageFormat::Png)?;
    println!("Saved PNG to: {:?}", png_path);

    // 2. Generate ICO file
    // Windows ICO format can be saved directly by image crate if resized
    let ico_path = assets_dir.join("icon.ico");
    let resized_256 = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
    resized_256.save_with_format(&ico_path, image::ImageFormat::Ico)?;
    println!("Saved ICO to: {:?}", ico_path);

    // 3. Also save a 64x64 and 32x32 brand logo for UI usage
    let logo_64 = img.resize_exact(64, 64, image::imageops::FilterType::Lanczos3);
    logo_64.save_with_format(assets_dir.join("brand_64.png"), image::ImageFormat::Png)?;

    println!("All icon assets successfully converted and saved!");
    Ok(())
}

