extern crate image;
use image::GenericImageView;
use std::fs::File;
use std::io::Write;
use std::fs;
use webbrowser;
use std::{thread, time::Duration};

// choose the right character for the pixel's intensity using the NTSC formula
pub fn get_ascii_char(r: u8, g: u8, b: u8) -> String{
    let characters = vec!["█", "▓", "▒", "░", " "];
    // max intensity is 255
    let intensity = r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114;
    // divide the intensity range into sections
    let divisor = 255./(characters.len()+1) as f32;
    // see which section the input falls on
    let mut index = ((intensity/divisor).floor()-1.) as usize;
    // return the character in that index
    if index >= characters.len(){
        index -= 1;
    }
    characters[index].to_string() 
}

// create a string consisting of spaces and █ characters. works for jpg and png.
// note: if printed in the terminal, the colors will be swapped since █ is white in the terminal but black everywhere else
pub fn img_to_asciistring(filename: &str, compressx: u32, compressy: u32, charamount: usize) -> String{
    // create empty string to store the final ascii
    let mut final_ascii = String::new();

    // load the image
    let img = image::open(filename).expect("bruh");

    // get width and height
    let (width, height) = img.dimensions();
    println!("Processing {}: {}x{}", filename, width, height);

    // iterate through the rows
    for y in 0..height {
        // only iterate through every n line to reduce the size
        if y % compressy == 0 {
            // iterate through the columns
            for x in 0..width {
                // only iterate through every n column to reduce the size
                if x % compressx == 0 {
                    // get the pixel's rgba value
                    let pixel = img.get_pixel(x, y);
                    // assign the rgba value to variables for comparison
                    let (r, g, b) = (pixel.0[0], pixel.0[1], pixel.0[2]);
                    // add a character
                    final_ascii += &get_ascii_char(r, g, b).repeat(charamount);

                }
            }
            final_ascii += "\n";
        }
    }
    final_ascii
}

// takes in a string of ASCII art and creates a html file.
pub fn write_html(name: &str, asciistring: String){
    // add the <pre> tag so that html won't delete the spaces
    let html_string = format!("{}\n{}\n{}", "<pre>", asciistring, "</pre>");
    // create the html's name
    let filename = format!("{}{}", name, ".html");
    // create the html file
    let file = File::create(filename);
    // write the string into the file
    let _ = match file{
        Ok(mut fileyes) => fileyes.write_all(html_string.as_bytes()),
        Err(_) => panic!("write_html failed!"),
    };
}

// get all files in a given folder
pub fn get_files(folder: &str) -> Vec<String>{
    // create an empty Vec to push img names into
    let mut img_name = Vec::new();
    // get path of the img folder
    let paths = fs::read_dir(folder).unwrap();
    for path in paths {
        img_name.push(path.unwrap().path().display().to_string())
    }
    img_name
}

// convert all (hopefully jpg) files in an img folder to html and put it in another folder
pub fn imgfold2htmlfold(imgpath: &str, htmlpath: &str, compressx: u32, compressy: u32){
    // iterate through filenames in the selected img folder
    for imagename in get_files(imgpath){
        // turn the image into ascii string
        let asciistring = img_to_asciistring(&imagename, compressx, compressy, 2);
        // set the html file name to [folder]\[image name] so that it goes in the correct folder
        let filename = format!("{}{}", htmlpath, &imagename[3..imagename.len()-4]);
        // write the html file
        write_html(&filename, asciistring);
    }
    print!("Completed!")
}

// open html files in a given folder, with a given delay in milliseconds
pub fn play_html(folder: &str, msdelay: u64){
    for path in get_files(folder){
        let _ = webbrowser::open(&path);
        thread::sleep(Duration::from_millis(msdelay))
    }
}

// turn binary into braille character
fn bin_to_braille(bin:&str) -> char{
    // turn binary to int
    let bin_to_int = isize::from_str_radix(bin, 2).unwrap();
    // now turn it into u32
    let bin_to_u32 = bin_to_int as u32;
    // return the character as braille
    char::from_u32(bin_to_u32 + 0x2800).unwrap()
}

// uses braille characters.
// *note: the braille characters are slightly smaller than blank ones, so it is a bit misshapened.
pub fn img_to_braillestring(filename: &str) -> String{
    // set the mapping of the braille dots, because it's mapped weirdly
    // |1|4|
    // |2|5|
    // |3|6|
    // |7|8|
    // -1 to each position for easy indexing
    let braillemap = [0, 2, 4, 1, 3, 5, 6, 7];

    // create empty string to store the final ascii
    let mut final_ascii: String = String::new();

    // load the image
    let img = image::open(filename).expect("bruh");

    // get width and height
    let (width, height) = img.dimensions();
    println!("Processing {}: {}x{}", filename, width, height);

    // iterate through the rows
    for y in 0..height {
        // every 4 rows
        if y % 4 == 0 {
            // iterate through the columns
            for x in 0..width {
                // every 2 columns and if the coordinates aren't out of range
                if x % 2 == 0 && x + 1 < width && y + 1 < height && y + 2 < height && y + 3 < height{
                    // new string to store the braille in binary form
                    let mut braille_bin = String::new();
                    // get position of pixels in a 2x4 grid
                    let pixelpos = [
                        (x, y), (x+1, y),
                        (x, y+1), (x+1, y+1),
                        (x, y+2), (x+1, y+2),
                        (x, y+3), (x+1, y+3),
                        ];
                    // go through the positions as it is writted in the braillemap vector
                    for i in braillemap{
                        let (currentx, currenty) = pixelpos[i];
                        // get rgba value
                        let pixel = img.get_pixel(currentx, currenty);
                        let (r, g, b) = (pixel.0[0], pixel.0[1], pixel.0[2]);
                        // NTSC intensity formula
                        let intensity = r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114;
                        // black pixel, add 1
                        if intensity < 127.5{
                            braille_bin = "1".to_string() + &braille_bin;
                        // white pixel, add 0
                        } else {
                            braille_bin = "0".to_string() + &braille_bin;
                        }
                    }
                    final_ascii += &bin_to_braille(&braille_bin).to_string();
                }
            }
            final_ascii += "\n";
        }
    }
    final_ascii
}

// like imgfold2htmlfold, but with the braille function instead
pub fn imgfold2htmlfoldbraille(imgpath: &str, htmlpath: &str){
    // iterate through filenames in the selected img folder
    for imagename in get_files(imgpath){
        // turn the image into ascii string
        let asciistring = img_to_braillestring(&imagename);
        // set the html file name to [folder]\[image name] so that it goes in the correct folder
        let filename = format!("{}{}", htmlpath, &imagename[3..imagename.len()-4]);
        // write the html file
        write_html(&filename, asciistring);
    }
    print!("Completed!")
}