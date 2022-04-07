
use serde::{Deserialize, Serialize};
use qrcode::QrCode;

use image;

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
// Specific use case is image with flat color background uploaded to bot to be turned into QR code
// ( also would like to do the typing notification thing while it's processing :3 )


//   Feature parity for current use: 
// Chromakey backgrounds color stuff
// 1000x1000 qr code size
// Works with bad aspect ratio files

// notes, default rn is H and data is about 160 alphanumeric characters? (~level 6)

fn main() {
    // TODO: take JSON RPC as input
    // write to /tmp
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
        min if min < 1 => qrcode::Version::Normal(1),
        max if max > 40 => qrcode::Version::Normal(40),
        _ => qrcode::Version::Normal(blob.version),
    };

    // TODO: minimum version selection check
    // use qrcode optimize to find min version
    // and make sure we're between that and 40
    // also probably that it fits in within 40
    
    // TODO: brightness and contrast are currently unused, likely no bounds check needed
    // contrast and brightness will be used to tune dithering/greyscale later

    // generate QR code =======================================================
    // TODO: actually use provided QR code version
    let code = QrCode::with_error_correction_level(blob.encoded_text, qrlevel).unwrap();
    //let code = QrCode::with_version(blob.text, qrversion, qrlevel).unwrap();
    //let code = QrCode::new(blob.text).unwrap();
	
    let qrcolors = code.to_colors();

    // TODO: make "modules" size adjustable
    // TODO: tweak module size, etc to be more readable
    // NOTE: aiming for ~1000x1000px 
    let imgscale: usize = 8;
    let imgwidth: u32 = (code.width() * imgscale) as u32;

    // make the QR mask =======================================================
    let mut qr_background: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>
                                    = image::ImageBuffer::new(imgwidth, imgwidth);
    let mut qr_mask: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>
                                    = image::ImageBuffer::new(imgwidth, imgwidth);

    for x in 0..imgwidth {
        for y in 0..imgwidth {
            // TODO: make alpha be floats instead of discrete?
            let modulex = (x as usize)/imgscale;
            let moduley = (y as usize)/imgscale;
            let index: usize = modulex + moduley*(code.width() as usize);
            let input = qrcolors[index];

            let color = if input == qrcode::Color::Dark { image::Rgb([0,0,0]) } 
                        else { image::Rgb([255,255,255])};
            
            let mut alpha: u8 = 0;
            
            // Centers of all Modules
            // we're making the outisdes of these transparent to show image below
            // we want to map |00001110000| kind of shape for mask
            // % is bad, we want to do vaguely smoothstep mirror
            // we need qr code start x and y (0, 0) and how wide it is (imgwidth)
            // mod number of pixels... then do cutoff
            // roughly: alpha value = some function on the distance from the center of the current module
            if (x as i32/2i32)%4 == 1 || (y as i32/2)%4 == 1 {alpha = 255 }
            if (x as i32/2i32)%4 == 2 || (y as i32/2)%4 == 2 {alpha = 255 }

            // Timing patterns etc.
            // some subset of these things are required for the code to be read
            // TODO: test subests to see what we can leave out
            if modulex == 6 || moduley == 6 { alpha = 255 } // timing pattern
            if modulex < 6 && moduley < 6 { alpha = 255 } // upper left
            if modulex >= code.width()-7 && moduley < 6 { alpha = 255 } // upper left
            if modulex < 6 && moduley >= code.width()-7 { alpha = 255 } // upper left
            // NOTE: ALIGNMENT_PATTERN_POSITIONS in qrcode has useful info for cleaning
            if modulex > code.width() - 10 && modulex < code.width() - 4
                && moduley > code.width() - 10 && moduley < code.width() - 4 { alpha = 255 }
            // TODO: add the version info, etc to the patterns we leave in
            
            // TODO: generate mask in a way that isn't embarassing
            qr_mask.put_pixel(x,y,image::Rgba([color[0], color[1], color[2], alpha]));
            qr_background.put_pixel(x,y,image::Rgba([color[0], color[1], color[2], 255]));
        }
    }

    // TODO: Add space for message text

    // grab base image and resize to our preferred output size
    // TODO: maintain input/output aspect ratio and center
    let mut baseimg = image::open(blob.input_filename).unwrap();

    // TODO: check for alpha to ignore chromakey
    // TODO: implement chromakey
    
    let resized = baseimg.resize(imgwidth, imgwidth, image::imageops::FilterType::CatmullRom);

    // TODO: quiet space around image
    // TODO: message below QR code
    let quietspace: u32 = 32;
    let messagespace: u32 = 55;

    // QR code <- image <- QR dot mask
    let totalwidth = imgwidth + quietspace;
    let totalheight = imgwidth + quietspace + messagespace;
    let mut output_image = 
            image::ImageBuffer::new(totalwidth, totalheight);

    // give background color
    for x in 0..totalwidth {
        for y in 0..totalheight {
            output_image.put_pixel(x,y,image::Rgba([44, 44, 44, 255]));
        }
    }

    image::imageops::overlay(&mut output_image, &qr_background, quietspace/2, quietspace/2);
    image::imageops::overlay(&mut output_image, &resized, quietspace/2, quietspace/2);
    image::imageops::overlay(&mut output_image, &qr_mask, quietspace/2, quietspace/2);
    image::imageops::overlay(&mut output_image, &qr_background, quietspace/2, quietspace/2);

    output_image.save(blob.output_filename).unwrap();	
}
