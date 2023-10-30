extern crate image;
use colored::*;
use image::GenericImageView;
use std::{borrow::Cow, fs, fs::File, io, io::Write, str, thread, time::{Duration, Instant}};
// ===================================================== helper functions =====================================================

// choose the right character for the pixel's intensity using the NTSC formula
pub fn get_ascii_char(r: u8, g: u8, b: u8, swap: bool) -> String {
    let mut characters = vec!["█", "▓", "▒", "░", " "];
    if swap {
        characters.reverse();
    }
    // max intensity is 255
    let intensity = r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114;
    // divide the intensity range into sections
    let divisor = 255. / (characters.len() + 1) as f32;
    // see which section the input falls on
    let mut index = ((intensity / divisor).floor() - 1.) as usize;
    // return the character in that index
    if index >= characters.len() {
        index -= 1;
    }
    characters[index].to_string()
}

// create a string consisting of spaces and █ characters.
pub fn img_to_asciistring(filename: &str, compress: u32, charamount: usize, swap: bool) -> String {
    let mut final_ascii = String::new();
    // load image
    let img = image::open(filename).expect("img_to_asciistring failed!");
    let (width, height) = img.dimensions();
    println!("Processing {}: {}x{}", filename, width, height);
    for y in 0..height {
        // only iterate through every n line to reduce the size
        if y % compress == 0 {
            let mut asciiline = String::new();
            for x in 0..width {
                // only iterate through every n column to reduce the size
                if x % compress == 0 {
                    // get the pixel at the (x, y) position and get an ascii character from its rgb value
                    let pixel: image::Rgba<u8> = img.get_pixel(x, y);
                    asciiline += &get_ascii_char(pixel.0[0], pixel.0[1], pixel.0[2], swap)
                        .repeat(charamount);
                }
            }
            final_ascii += &format!("{}{}", asciiline.trim_end(), "\n");
        }
    }
    final_ascii
}

// separate function as the output has to be in vector form instead
pub fn img_to_cblock(filename: &str, compress: u32) -> Vec<Vec<ColoredString>> {
    let mut final_vec = Vec::new();
    // load image
    let img = image::open(filename).expect("img_to_cblock failed!");
    let (width, height) = img.dimensions();
    println!("Processing {}: {}x{}", filename, width, height);
    for y in 0..height {
        if y % compress == 0 {
            let mut asciiline = Vec::new();
            for x in 0..width {
                if x % compress == 0 {
                    // get the pixel at the (x, y) position and get an ascii character from its rgb value
                    let pixel: image::Rgba<u8> = img.get_pixel(x, y);
                    let colored_string = "███".truecolor(pixel.0[0], pixel.0[1], pixel.0[2]);
                    asciiline.push(colored_string)
                }
            }
            final_vec.push(asciiline);
        }
    }
    final_vec
}

// turn binary into braille character
pub fn bin_to_braille(bin: &str, whitespace: bool) -> char {
    if bin == "00000000" {
        if whitespace {
            return ' ';
        } else {
            return '⡀';
        }
    }
    // turn binary to int
    let bin_to_int = isize::from_str_radix(bin, 2).unwrap();
    // now turn it into u32
    let bin_to_u32 = bin_to_int as u32;
    // return the character as braille
    char::from_u32(bin_to_u32 + 0x2800).unwrap()
}

// create a string consisting of braille characters. uses whitespaces if specified so.
pub fn img_to_braillestring(filename: &str, compress: u32, whitespace: bool, swap: bool) -> String {
    // set the mapping of the braille dots
    let braillemap = [0, 2, 4, 1, 3, 5, 6, 7];
    let mut final_ascii: String = String::new();
    // load the image
    let img = image::open(filename).expect("img_to_braillestring failed!");
    let (width, height) = img.dimensions();
    println!("Processing {}: {}x{}", filename, width, height);
    for y in 0..height {
        // every 4*n rows
        if y % (4 * compress) == 0 {
            let mut asciiline = String::new();
            for x in 0..width {
                // every 2*n columns and if the coordinates aren't out of range
                if x % (2 * compress) == 0
                    && x + compress < width
                    && y + compress < height
                    && y + 2 * compress < height
                    && y + 3 * compress < height
                {
                    let mut braille_bin = String::new();
                    // get position of pixels in a 2x4 grid
                    let pixelpos = [
                        (x, y),
                        (x + compress, y),
                        (x, y + compress),
                        (x + compress, y + compress),
                        (x, y + 2 * compress),
                        (x + compress, y + 2 * compress),
                        (x, y + 3 * compress),
                        (x + compress, y + 3 * compress),
                    ];
                    // follow the mapped value
                    for i in braillemap {
                        let (currentx, currenty) = pixelpos[i];
                        let pixel = img.get_pixel(currentx, currenty);
                        // NTSC intensity formula
                        let intensity = pixel.0[0] as f32 * 0.299
                            + pixel.0[1] as f32 * 0.587
                            + pixel.0[2] as f32 * 0.114;
                        // black pixel = 1
                        if swap {
                            if intensity < 127.5 {
                                braille_bin = "0".to_string() + &braille_bin;
                            // white pixel = 0
                            } else {
                                braille_bin = "1".to_string() + &braille_bin;
                            }
                        } else {
                            if intensity < 127.5 {
                                braille_bin = "1".to_string() + &braille_bin;
                            // white pixel = 0
                            } else {
                                braille_bin = "0".to_string() + &braille_bin;
                            }
                        }
                    }
                    asciiline += &bin_to_braille(&braille_bin, whitespace).to_string();
                }
            }
            final_ascii += &(asciiline.trim_end().to_owned() + "\n");
        }
    }

    final_ascii
}

// get file names in a given folder
pub fn get_files(folder: &str) -> Vec<String> {
    let mut img_name = Vec::new();
    let paths = fs::read_dir(folder).unwrap();
    for path in paths {
        img_name.push(path.unwrap().path().display().to_string())
    }
    img_name
}

// takes in a string of ascii art and creates a html file.
pub fn write_html(name: &str, asciistring: String) {
    // add the <pre> tag so that html won't delete the spaces
    let html_string = format!("{}\n{}\n{}", "<pre>", asciistring, "</pre>");
    // write the string into the .html file
    let filename = format!("{}.html", name);
    let file = File::create(filename);
    let _ = match file {
        Ok(mut fileyes) => fileyes.write_all(html_string.as_bytes()),
        Err(e) => panic!("write_html failed: {}", e),
    };
}

// takes in a string of ascii art and creates a txt file.
pub fn write_txt(name: &str, asciistring: String) {
    // write the string into the .txt file
    let filename = format!("{}.txt", name);
    let file = File::create(filename);
    let _ = match file {
        Ok(mut fileyes) => fileyes.write_all(asciistring.as_bytes()),
        Err(e) => panic!("write_txt failed: {}", e),
    };
}

// Used to put files into a folder
pub fn get_destination(filename: &Cow<'_, str>, output: &str) -> String {
    // get file name without the folder path
    let split_name = filename.split("\\").collect::<Vec<&str>>();
    let nofoldname = split_name[split_name.len() - 1].split(".").collect::<Vec<&str>>();
    // assign destination folder
    format!("{}\\{}", output, &nofoldname[0])
}

// ===================================================== functions =====================================================

// takes a folder and turns the entire folder into html or txt file and put it into the specified folder.
pub fn imgfold2asciifold(
    imgpath: &str,
    asciipath: &str,
    whitespace: bool,
    chartype: &str,
    filetype: &str,
) {
    for imagename in get_files(imgpath) {
        let mut asciistring = String::new();
        if chartype == "block" {
            asciistring = img_to_asciistring(&imagename, 1, 2, false);
        } else if chartype == "braille" {
            asciistring = img_to_braillestring(&imagename, 1, whitespace, false);
        }
        // name = [output path][image name without path], no .html as write_html already adds the .html
        let split_name = imagename.split("\\").collect::<Vec<&str>>();
        let filename = format!("{}\\{}", asciipath, split_name[split_name.len() - 1]);
        if filetype == "html" {
            write_html(&filename, asciistring)
        } else if filetype == "txt" {
            write_txt(&filename, asciistring)
        }
    }
}

// compresses the image and prints it as ascii art.
pub fn write_term(filename: &str, chartype: &str, whitespace: bool, colored: bool) {
    let mut compress = 1;
    let img = image::open(filename).expect("write_term failed!");
    let (width, _) = img.dimensions();
    if chartype == "braille" {
        while width / compress >= 400 {
            compress += 1;
        }
        let asciistring = img_to_braillestring(&filename, compress, whitespace, true);
        println!("{}", asciistring);
    } else if chartype == "block" && colored {
        while width / compress >= 67 {
            compress += 1;
        }
        let block_vec = img_to_cblock(filename, compress);
        for line in block_vec {
            for char in line {
                print!("{}", char)
            }
            println!("")
        }
    } else if chartype == "block" && !colored {
        while width / compress >= 67 {
            compress += 1;
        }
        let asciistring = img_to_asciistring(&filename, compress, 3, true);
        println!("{}", asciistring);
    }
}

// print all compressed images in a folder into the terminal as ascii art
pub fn imgfold2term(imgpath: &str, chartype: &str, whitespace: bool, colored: bool, msdelay: u64) {
    let files = get_files(imgpath);
    let mut compress = 1;
    let img = image::open(&files[0]).expect("imgfold2term failed!");
    let (width, _) = img.dimensions();
    if chartype == "block" {
        while width / compress >= 67 {
            compress += 1;
        }
    } else if chartype == "braille" {
        while width / compress >= 400 {
            compress += 1;
        }
    }
    if chartype == "braille" || !colored {
        let mut frames = Vec::new();
        for imagename in files {
            let mut asciistring = String::new();
            if chartype == "block" {
                asciistring = img_to_asciistring(&imagename, compress, 3, true);
            } else if chartype == "braille" {
                asciistring = img_to_braillestring(&imagename, compress, whitespace, true);
            }
            frames.push(asciistring);
        }
        println!("Image loading complete, press Enter to begin playing");
        let mut _buffer = String::new();
        let _ = io::stdin().read_line(&mut _buffer);
        for frame in frames {
            let start_time = Instant::now();
            println!("{frame}");
            let end_time = Instant::now();
            let duration = end_time.duration_since(start_time);
            let processtime = duration.as_millis();
            thread::sleep(Duration::from_millis(msdelay - processtime as u64))
        }
    } else if chartype == "block" && colored {
        let mut frames = Vec::new();
        for imagename in files{
            let block_vec = img_to_cblock(&imagename, compress);
            frames.push(block_vec);
        }
        println!("Image loading complete, press Enter to begin playing");
        let mut _buffer = String::new();
        let _ = io::stdin().read_line(&mut _buffer);
        for frame in frames {
            let start_time = Instant::now();
            for row in frame{
                for pixel in row {
                    print!("{pixel}")
                }
                println!("")
            }
            let end_time = Instant::now();
            let duration = end_time.duration_since(start_time);
            let processtime = duration.as_millis();
            thread::sleep(Duration::from_millis(msdelay - processtime as u64))
        }
    }
}

pub fn write_chtml(name: &str, output: &str){
    let img = image::open(name).expect("write_chtml failed!");
    let (width, height) = img.dimensions();
    let mut final_string = String::from("<pre>\n");
    println!("Processing {}: {}x{}", name, width, height);
    for y in 0..height{
        for x in 0..width{
            let pixel: image::Rgba<u8> = img.get_pixel(x, y);
            let (r, g, b) = (pixel.0[0], pixel.0[1], pixel.0[2]);
            for _ in 0..2{
                final_string += &format!("<font color='#{:02x}{:02x}{:02x}'>█</font>", r, g, b)
            }
        }
        final_string += "\n"
    }
    final_string += "</pre>";
    let filename = format!("{output}.html");
    let file = File::create(filename);
    let _ = match file {
        Ok(mut fileyes) => fileyes.write_all(final_string.as_bytes()),
        Err(e) => panic!("write_html failed: {}", e),
    };
}

pub fn imgfold2chtml(imgpath: &str, asciipath: &str){
    let files = get_files(imgpath);
    println!("{files:?}");
    for imagename in files{
        let split_name = imagename.split("\\").collect::<Vec<&str>>();
        let filename = format!("{}\\{}", asciipath, split_name[split_name.len() - 1]);
        write_chtml(&imagename, &filename)
    }
}