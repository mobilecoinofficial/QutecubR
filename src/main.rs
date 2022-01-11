use qrcode::QrCode;

use image;
use std::env;

fn main() {
    // deal with arguments
    // TODO(Opal): make a proper API
    let text_input = env::args().nth(1)
        .expect("Expected a code to output.");
    let path_input = env::args().nth(2)
        .expect("Expected a filename to output to.");

    // generate QR code
    // TODO(Opal): remove dependency on qrcode
    let code = QrCode::new(text_input).unwrap();
    
    let string = code.render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    println!("{}", string);
	
    let qrcolors = code.to_colors();

    // hardcoding "modules" as 6x6 with center two b&w for now
    // TODO(Opal): make "modules" size adjustable
    let imgscale: usize = 6;
    let imgwidth: u32 = (code.width() * imgscale) as u32;
    println!("code.width: {}, code.width*imgwidth: {}", code.width(), code.width()*code.width());
    println!("imgwidth: {}, imgwidth*imgwidth: {}", imgwidth, imgwidth*imgwidth);

    let mut imgbuf = image::ImageBuffer::new(imgwidth, imgwidth);

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
            // NOTE(Opal): ALIGNMENT_PATTERN_POSITIONS in qrcode has useful info, can make this cleaner
            let mut alpha: u8 = 0;
            if (x as i32/2i32)%3 == 1 && (y as i32/2)%3 == 1 {alpha = 255 } // centers
            if modulex == 6 || moduley == 6 { alpha = 255 } // timing pattern
            if modulex < 6 && moduley < 6 { alpha = 255 } // upper left
            if modulex >= code.width()-7 && moduley < 6 { alpha = 255 } // upper left
            if modulex < 6 && moduley >= code.width()-7 { alpha = 255 } // upper left
            if modulex > code.width() - 10 && modulex < code.width() - 4
                && moduley > code.width() - 10 && moduley < code.width() - 4 { alpha = 255 }
            

            imgbuf.put_pixel(x,y,image::Rgba([color[0], color[1], color[2], alpha]));

        }
    }

    
    let mut baseimg = image::open("underlay.png").unwrap();
    image::imageops::overlay(&mut baseimg, &imgbuf, 0, 0);



    //baseimg.save(path_input).unwrap();
    imgbuf.save(path_input).unwrap();

    // whatever bullshit to convert QrCode to imgbuf

	
	
}
