
use serde::{Deserialize, Serialize};
use serde_json::Result;
use qrcode::QrCode;

use image;
use std::env;

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

// would like to be drop-in replacement from 
// https://github.com/mobilecoinofficial/forest/blob/main/mobfriend/mobfriend.py#L118
// also would like to do the typing notification thing while it's processing :3

fn main() {
    // TODO: take JSON blob as argument
    // TODO: change JSON blob to JSON RPC
    // we'll pretend we were passed this for now:
    let testjson = r#"
            {
                "encoded_text": "https://mobilecoin.com/",
                "message_text": "Check out MobileCoin!",
                "colorized": true,
                "chromakey": true,
                "version": 1,
                "level": "H",
                "contrast": 1.0,
                "brightness": 1.0,
                "input_filename": "data/input.png",
                "output_filename": "data/output.png"
            }"#;
    
    // TODO: proper default values
    let blob: Blob = serde_json::from_str(testjson).unwrap();

    // pick level, default H
    let qrlevel = match blob.level.as_str() {
       "L" => qrcode::EcLevel::L,
       "M" => qrcode::EcLevel::M,
       "Q" => qrcode::EcLevel::Q,
       "H" => qrcode::EcLevel::H,
       _ => qrcode::EcLevel::H,
    };

    // pick version, up to 40
    let qrversion = match blob.version {
        v if v < 1 => qrcode::Version::Normal(1),
        v if v > 40 => qrcode::Version::Normal(40),
        _ => qrcode::Version::Normal(blob.version),
    };

    // TODO: minimum version selection check
    // use qrcode optimize to find min version
    // and make sure we're between that and 40
    // also probably that it fits in within 40
    
    // TODO: brightness and contrast are currently unused, likely no bounds check needed

    // generate QR code =======================================================
    // TODO: actually use provided QR code version
    let code = QrCode::with_error_correction_level(blob.encoded_text, qrlevel).unwrap();
    //let code = QrCode::with_version(blob.text, qrversion, qrlevel).unwrap();
    //let code = QrCode::new(blob.text).unwrap();
	
    let qrcolors = code.to_colors();

    // TODO: make "modules" size adjustable
    // TODO: tweak module size, etc to be more readable
    let imgscale: usize = 6;
    let imgwidth: u32 = (code.width() * imgscale) as u32;

    // make the QR mask =======================================================
    let mut qr_mask = image::ImageBuffer::new(imgwidth, imgwidth);

    for x in 0..imgwidth {
        for y in 0..imgwidth {
            let modulex = (x as usize)/imgscale;
            let moduley = (y as usize)/imgscale;
            let index: usize = modulex + moduley*(code.width() as usize);
            let input = qrcolors[index];

            let color = if input == qrcode::Color::Dark { image::Rgb([0,0,0]) } 
                        else { image::Rgb([255,255,255])};
            
            // make the "modules" transparent on outsides for the overlay to work
            // keeping alignement and timing patterns opaque until we can test properly
            // NOTE: ALIGNMENT_PATTERN_POSITIONS in qrcode has useful info for cleaning
            let mut alpha: u8 = 0;
            
            if (x as i32/2i32)%3 == 1 && (y as i32/2)%3 == 1 {alpha = 255 } // centers
            if modulex == 6 || moduley == 6 { alpha = 255 } // timing pattern
            if modulex < 6 && moduley < 6 { alpha = 255 } // upper left
            if modulex >= code.width()-7 && moduley < 6 { alpha = 255 } // upper left
            if modulex < 6 && moduley >= code.width()-7 { alpha = 255 } // upper left
            if modulex > code.width() - 10 && modulex < code.width() - 4
                && moduley > code.width() - 10 && moduley < code.width() - 4 { alpha = 255 }
            
            // TODO: generate mask in a way that isn't embarassing
            qr_mask.put_pixel(x,y,image::Rgba([color[0], color[1], color[2], alpha]));
        }
    }

    // TODO: Add space for message text

    // grab base image and resize to our preferred output size
    // TODO: maintain input/output aspect ratio and center
    let baseimg = image::open(blob.input_filename).unwrap();
    // TODO: check for alpha to ignore chromakey
    // TODO: implement chromakey
    let resized = baseimg.resize(imgwidth, imgwidth, image::imageops::FilterType::CatmullRom);

    // TODO: quiet space around image
    // TODO: message below QR code
    // QR code <- image <- QR dot mask
    let quietspace: u32 = 32;
    let messagespace: u32 = 55;

    let mut output_image = 
            image::ImageBuffer::new(imgwidth + quietspace, imgwidth + quietspace + messagespace);
    image::imageops::overlay(&mut output_image, &qr_mask, quietspace/2, quietspace/2);
    image::imageops::overlay(&mut output_image, &resized, quietspace/2, quietspace/2);
    image::imageops::overlay(&mut output_image, &qr_mask, quietspace/2, quietspace/2);

    output_image.save(blob.output_filename).unwrap();	
}
