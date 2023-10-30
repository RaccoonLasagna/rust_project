use clap::{App, Arg};
use img_to_ascii::*;

fn main() {
    // Define the command line application and its arguments using Clap
    let matches = App::new("img_to_ascii")
        .version("1.3")
        .author("Kongfah Sangchaisirisak")
        .about("An image to ASCII conversion program.")
        .arg(
            Arg::with_name("filename")
                .value_name("filename")
                .help("Input the name of the image file in JPG or PNG")
                .index(1)
                .required(true)
        )
        .arg(
            Arg::with_name("block")
                .short("l")
                .long("block")
                .help("Creates ASCII art using block elements")
                .conflicts_with("braille"),
        )
        .arg(
            Arg::with_name("braille")
                .short("r")
                .long("braille")
                .help("Creates ASCII art using braille characters")
        )
        .arg(
            Arg::with_name("html")
                .short("h")
                .long("html")
                .help("Outputs the file as an html file")
                .conflicts_with("text"),
        )
        .arg(
            Arg::with_name("text")
                .short("t")
                .long("text")
                .help("Outputs the file as a txt file"),
        )
        .arg(
            Arg::with_name("whitespace")
                .short("w")
                .long("whitespace")
                .help("Uses whitespaces for empty pixels for braille ASCII art. Does not do anything for block.\nWarning: may cause misalignment"),
        )
        .arg(
            Arg::with_name("colored")
                .short("c")
                .long("colored")
                .help("Colors html and terminal output. Does not do anything for braille and txt output.\nWarning: will make the conversions slower"),
        )
        .get_matches();
    
    if !(matches.is_present("block") || matches.is_present("braille")){
        eprintln!(
            "error: --braille or --block is required

USAGE:
    img_to_ascii.exe <filename> --block
    OR
    img_to_ascii.exe <filename> --braille"
        );
        std::process::exit(1);
    }

    // if input has a "." in it, it's not a folder.
    let filename = matches.value_of_lossy("filename").unwrap();
    let split_filename = filename.split(".").collect::<Vec<&str>>();
    let mut folder = true;
    if split_filename.len() > 1 {
        folder = false
    }

    // (filename, folder, block, braille, html, text, whitespace)
    let options = (
        matches.value_of_lossy("filename").unwrap(),
        folder,
        matches.is_present("block"),
        matches.is_present("braille"),
        matches.is_present("html"),
        matches.is_present("text"),
        matches.is_present("whitespace"),
        matches.is_present("colored"),
    );
    let destination = get_destination(&filename, "output");
    match options{
        // single img, block, html, uncolored
        (filename, false, true, false, true, false, _, false) => {
            write_html(&destination, img_to_asciistring(&filename, 1, 2, false));
        }
        // single img, block, html, colored
        (filename, false, true, false, true, false, _, true) => {
            write_chtml(&filename, &destination);
        }
        // single img, block, txt
        (filename, false, true, false,  false, true, _, _) => {
            write_txt(&destination, img_to_asciistring(&filename, 1, 2, false));
        }
        // single img, block, terminal
        (filename, false, true, false, false, false, _, colored) => {
            write_term(filename.as_ref(), "block", false, colored);
        }
        // single img, braille, html
        (filename, false, false, true, true, false, whitespace, _) => {
            write_html(&destination, img_to_braillestring(filename.as_ref(), 2, whitespace, false))
        }
        // single img, braille, txt
        (filename, false, false, true, false, true, whitespace, _) => {
            write_txt(&destination, img_to_braillestring(filename.as_ref(), 1, whitespace, false))
        }
        // single img, braille, terminal
        (filename, false, false, true, false, false, whitespace, _) => {
            write_term(filename.as_ref(), "braille", whitespace, false);
        }
        // folder, block, html, uncolored
        (filename, true, true, false, true, false, _, false) => {
            imgfold2asciifold(&filename, "output", false, "block", "html")
        }
        // folder, block, html, colored
        (filename, true, true, false, true, false, _, true) => {
            imgfold2chtml(&filename, "output");
        }
        // folder, block, txt
        (filename, true, true, false, false, true, _, _) => {
            imgfold2asciifold(&filename, "output", false, "block", "txt")
        }
        // folder, block, terminal
        (filename, true, true, false, false, false, _, colored) => {
            imgfold2term(&filename, "block", false, colored, 200);
        }
        // folder, braille, html
        (filename, true, false, true, true, false, whitespace, _) => {
            imgfold2asciifold(&filename, "output", whitespace, "braille", "html")
        }
        // folder, braille, txt
        (filename, true, false, true, false, true, whitespace, _) => {
            imgfold2asciifold(&filename, "output", whitespace, "braille", "txt")
        }
        // folder, braille, terminal
        (filename, true, false, true, false, false, whitespace, _) => {
            imgfold2term(&filename, "braille", whitespace, false, 200);
        }

        // For catching cases I may have missed
        _ => println!("{:?} case not covered, oops", options)
    }
    println!("Completed!")
}