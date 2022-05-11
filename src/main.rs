// QutecumbR - a cute QR code generator in rust.
use serde::{Deserialize, Serialize}; // for JSON
use qrcode::QrCode;    // for QR code generation
use image;    // for image operations and output
use std::time::{Duration, Instant};

// JSON blob format we're expecting as input
#[derive(Debug, Serialize, Deserialize)]
struct Blob {
    encoded_text: String,
    message_text: String,
    colorized: bool,
    chromakey: bool,
    version: i16,
    level: String,
    contrast: f32,
    brightness: f32,
    input_filename: String,
    output_filename: String,
}

// Overall, we would like to be drop-in replacemen entering from mobfriend line 118:
// https://github.com/mobilecoinofficial/forest/blob/main/mobfriend/mobfriend.py#L118
// specifically: image with flat color background uploaded to bot along with text
//      to be turned into a QR code with some image compositing and some text

// feature parity checklist: 
// 1. chromakey background color
// 2. roughly 1000x1000 image output size
// 3. elegantly handles aspect ratio mismatch
// NOTE: currently, defaults are H and data is about 160 alphanumeric characters? (~level 6)

fn main() {
    // should be entering from mobfriend ~111: _actually_build_wait_and_send_qr()
    // https://github.com/mobilecoinofficial/forest/blob/main/mobfriend/mobfriend.py#L111

    // TODO: actually take JSON RPC as input!
    // TODO: write file out to /tmp
    // We'll pretend we were passed this for now:
    let testjson = r#"
            {
                "encoded_text": "https://signal.me/#p/+12692304655",
                "message_text": "",
                "colorized": true,
                "chromakey": true,
                "version": 1,
                "level": "H",
                "contrast": 1.0,
                "brightness": 1.0,
                "input_filename": "data/whispr_avatar.png",
                "output_filename": "data/output.png"
            }"#;
    
    // Parse the JSON =========================================================
    let settings: Blob = serde_json::from_str(testjson).unwrap();

    // pick level, default H
    let qrlevel = match settings.level.as_str() {
       "L" => qrcode::EcLevel::L,
       "M" => qrcode::EcLevel::M,
       "Q" => qrcode::EcLevel::Q,
       "H" => qrcode::EcLevel::H,
       _ => qrcode::EcLevel::H,
    };

    // pick version, max 40
    // TODO: minimum version selection check
    // use qrcode optimize to find min version
    // and make sure we're between that and 40
    // also probably that it fits in within 40
    // NOTE: this isn't used yet
    let qrversion = match settings.version {
        min if min < 1 => qrcode::Version::Normal(1),
        max if max > 40 => qrcode::Version::Normal(40),
        _ => qrcode::Version::Normal(settings.version),
    };

    // TODO: brightness and contrast are currently unused, likely no bounds check needed
    // contrast and brightness will be used to tune dithering/greyscale later

    // Generate QR code =======================================================
    // let code = QrCode::new(blob.text).unwrap();
    // let code = QrCode::with_version(blob.text, qrversion, qrlevel).unwrap();
    // TODO: actually use provided QR code version
    let code = QrCode::with_error_correction_level(settings.encoded_text, qrlevel).unwrap();
    let qrcolors = code.to_colors(); // colors in this case is pixels?

    // Render the QR mask =====================================================
    // TODO: make "modules" size adjustable
    // TODO: tweak module size, etc to be more readable
    // NOTE: aiming for ~1000x1000px 
    let modulepx: usize = 16;
    let qrcodepx: u32 = (code.width() * modulepx) as u32;

    let mut qr_background: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>
                                    = image::ImageBuffer::new(qrcodepx, qrcodepx);
    let mut qr_mask: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>
                                    = image::ImageBuffer::new(qrcodepx, qrcodepx);

    for x in 0..qrcodepx {
        for y in 0..qrcodepx {
            let mx = (x as usize)/modulepx; // module x
            let my = (y as usize)/modulepx; // module y
            let color = if qrcolors[mx + my*(code.width() as usize)]
                            == qrcode::Color::Dark { image::Rgb([0,0,0]) } 
                        else { image::Rgb([255,255,255])};
            
            // Warning: maths.
            // Making the center of modules opaque and leaving the rest trans
            // (using smoothstep to make it resolution independent)
            
            // we need:  left side of module, Right side of module, pixel center
            // (and same for the vertical)
            // roughly: alpha value = some function on the distance from the center of the current module
            // alpha = smoothstep(x);
            let ch: f32 = ((x as f32)/(modulepx as f32))%1.0; // horizontal pixel center relative to center of module
            let cv: f32 = ((y as f32)/(modulepx as f32))%1.0; // vertical pixel center relative to center of module
            let ta = 0.22; // tuning A
            let tb = 0.06; // tuning B
            let ah = ((255 as f32) * (smoothstep((0.5-ch).abs(), ta, ta-tb))) as u8;
            let av = ((255 as f32) * (smoothstep((0.5-cv).abs(), ta, ta-tb))) as u8;
            let mut alpha: u8 = ah.min(av); // just get the min alpha of these two

            // Special Cases for timing patterns
            // TODO: test subests to see what subset we can comfortably leave out
            if mx == 6 || my == 6 { alpha = 255 }       // timing pattern
            if mx <= 6 && my <= 6 { alpha = 255 }               // upper left
            if mx >= code.width()-7 && my <= 6 { alpha = 255 } // upper right
            if mx <= 6 && my >= code.width()-7 { alpha = 255 } // lower left
            
            // TODO: add the version info, etc to the patterns we leave in
            // NOTE: ALIGNMENT_PATTERN_POSITIONS in qrcode has useful information
            if mx > code.width() - 10 && mx < code.width() - 4
                && my > code.width() - 10 && my < code.width() - 4 { alpha = 255 }
            
            // TODO: generate mask(s) in a way that isn't embarassing
            //alpha = 255;
            qr_mask.put_pixel(x, y, image::Rgba([color[0], color[1], color[2], alpha]));
            qr_background.put_pixel(x, y, image::Rgba([color[0], color[1], color[2], 255]));
        }
    }

    // Load image to be composited ============================================
    // TODO: implement chromakey (might want to keep the color as background color)
    // TODO: check for alpha to ignore chromakey
    
    // TODO: maintain input/output aspect ratio
    let ii_resized = image::open(settings.input_filename).unwrap()
            .resize(qrcodepx, qrcodepx, image::imageops::FilterType::CatmullRom);

    println!("Resolution: {}x{}", qrcodepx, qrcodepx);

    let quietspace: u32 = (modulepx as u32)*4; // pixel skirt beyond each edge of QR
    let messagespace: u32 = 0; // additional space below image
    // TODO: actually render text messages below QR code
    // NOTE: using fonts is a tricky copyright thing, potentially.
    //   we got around this in games by rendering fonts into bitmaps offline then using those

    // QR code <- image <- QR dot mask
    let totalwidth = qrcodepx + quietspace*2;
    let totalheight = qrcodepx + quietspace*2 + messagespace;
    let mut output_image =  image::ImageBuffer::new(totalwidth, totalheight);

    // give background color
    for x in 0..totalwidth {
        for y in 0..totalheight {
            //output_image.put_pixel(x,y,image::Rgba([44, 44, 44, 255]));
            output_image.put_pixel(x,y,image::Rgba([58, 118, 240, 255]));
        }
    }

    image::imageops::overlay(&mut output_image, &qr_background, quietspace, quietspace);
    image::imageops::overlay(&mut output_image, &ii_resized, quietspace, quietspace);
    image::imageops::overlay(&mut output_image, &qr_mask, quietspace, quietspace); 

    output_image.save(settings.output_filename).unwrap();
}

fn smoothstep(x: f32, a: f32,  b: f32) -> f32 {
    return (1 as f32).min((0 as f32).max((x-a)/(b-a)));
}
