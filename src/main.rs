
use serde::{Deserialize, Serialize};
use serde_json::Result;
use qrcode::QrCode;

use image;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    code: String,
    input_filename: String,
    output_filename: String,
}

// would like to be drop-in replacement from https://github.com/mobilecoinofficial/forest/blob/main/mobfriend/mobfriend.py#L51

fn main() {
    // TODO: take JSON settings as argument
    // we'll pretend we were passed this for now
    let testjson = r#"
            {
                "code": "https://mobilecoin.com/",
                "input_filename": "data/input.png",
                "output_filename": "data/output.png"
            }"#;

    let settings: Settings = serde_json::from_str(testjson).unwrap();
    println!("{:?}", settings);

    // generate QR code ============================================
    let code = QrCode::new(settings.code).unwrap();
    
    let string = code.render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    println!("{}", string);
	
    let qrcolors = code.to_colors();

    // hardcoding "modules" as 6x6 with center two b&w for now
    // TODO: make "modules" size adjustable
    // TODO: tweak module size, etc to be more readable
    let imgscale: usize = 6;
    let imgwidth: u32 = (code.width() * imgscale) as u32;
    println!("code.width: {}, code.width*imgwidth: {}", code.width(), code.width()*code.width());
    println!("imgwidth: {}, imgwidth*imgwidth: {}", imgwidth, imgwidth*imgwidth);

    // make the qr mask to apply over image (frames)
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
            // we want to keep alignment and timing patterns fully opaque
            // NOTE: ALIGNMENT_PATTERN_POSITIONS in qrcode has useful info, can make this cleaner
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

    // grab base image and resize to our preferred output size
    // TODO: maintain input/output aspect ratio and center
    let baseimg = image::open(settings.input_filename).unwrap();
    let mut resized = baseimg.resize(imgwidth, imgwidth, image::imageops::FilterType::CatmullRom);

    // throw overlay on the base image... this fundamentally what we're doing here
    image::imageops::overlay(&mut resized, &qr_mask, 0, 0);

    resized.save(settings.output_filename).unwrap();	
}
