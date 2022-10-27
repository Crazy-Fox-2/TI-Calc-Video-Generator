use crate::helper::macros::strcat;
use std::env;
use getopts::Occur;
use args::Args;


pub struct VArgs {
    pub vid_file: String,
    pub vid_folder: String,
    pub out: String,
    pub name: String,
    pub start: usize,
    pub dur: usize,
    pub calc_fps: f64,
    pub mute: bool,
    pub dither: char,
    pub audoff: i32,
    pub cycle_limit: usize,
    pub dbg_out: bool,
}


pub fn getargs() -> Result<VArgs, String> {
    let mut args = Args::new("ti-audvid-convert", "Comverts a given video and transforms it into an application to be played back on a TI-83+SE or TI-84+(SE) calculator");
    
    args.option("v", "video", "Source video file", "VID", Occur::Optional, Some("".to_string()));
    args.option("f", "folder", "Source/Dest video folder", "FOLDER", Occur::Optional, Some("".to_string()));
    args.option("o", "out", "Output application file", "OUT", Occur::Req, None);
    args.option("n", "name", "Output application name (8 chars max)", "NAME", Occur::Req, None);
    args.option("d", "duration", "How many calculator frames to convert from the video, omit for entire video", "DUR", Occur::Optional, Some("0".to_string()));
    args.option("s", "start", "Which calculator frame to start on, default first frame", "ST", Occur::Optional, Some("0".to_string()));
    args.option("p", "fps", "Change framerate of video on the calculator, default 20fps, try to keep this close to 20 (19-21)", "FPS", Occur::Optional, Some("20.0".to_string()));
    args.option("a", "audoff", "Audio offset, a greater value means audio will play sooner, negative values allowed", "AUDOFF", Occur::Optional, Some("0".to_string()));
    args.option("t", "dither", "The dither mode, either f for floyd-steinburg or o for ordered, deafualt=o", "DITHER", Occur::Optional, Some("o".to_string()));
    args.option("c", "cycle_limit", "Adjust maximum cycle cost per frame. Intended for debug/demonstrational use, try to avoid", "CT", Occur::Optional, Some("120000".to_string()));
    
    args.flag("m", "mute", "Flag - shuts me up");
    args.flag("g", "debug", "Flag - output debug files during convert");
    args.flag("h", "help", "Flag - Print this help message");

    /*
    // Input options
    args.option("a", "answer", "The solution to today's Wordle", "ANS", Occur::Req, None);
    args.option("i", "img", "The input image to convert", "IMG", Occur::Req, None);
    args.option("d", "dict", "Optional - Dictionary of allowed words, default is the dict.txt provided", "DICT", Occur::Optional, Some(String::from("dict.txt")));
    // Transform options
    args.option("t", "trans", "Optional - Transform steps, see readme, default is no transformation", "TRANS", Occur::Optional, Some(String::from(" ")));
    // Output options
    args.option("o", "out", "Output file", "OUT", Occur::Req, None);
    args.option("u", "hint-out", "Optional - Hint output type, a=all hints, f=first hint, r=random hint, default is first", "TYPE", Occur::Optional, Some(String::from("f")));
    args.flag("n", "nope", "Flag - When included then no output will be writen if any rows fail to find a hint");
    args.flag("h", "help", "Flag - Print this help message");
    */

    // Check for help argument
    let mut done_help = false;
    let v: Vec<String> = env::args().collect();
    for s in v {
        if s.eq("-h") || s.eq("--help") {
            println!("{}", args.full_usage());
            done_help = true;
        }
    }
    
    match args.parse_from_cli() {
        Ok(()) => Ok(VArgs {
            vid_file: {match args.value_of::<String>("video") {
                Ok(s) => s,
                Err(_) => "".to_string(),
            }},
            vid_folder: {match args.value_of::<String>("folder") {
                Ok(s) => strcat!(s, "/"),
                Err(_) => "".to_string(),
            }},
            out: args.value_of::<String>("out").unwrap(),
            name: args.value_of::<String>("name").unwrap(),
            dur: args.value_of::<usize>("duration").unwrap(),
            start: args.value_of::<usize>("start").unwrap(),
            calc_fps: args.value_of::<f64>("fps").unwrap(),
            mute: args.value_of::<bool>("mute").unwrap(),
            dither: args.value_of::<String>("dither").unwrap().chars().next().unwrap(),
            audoff: args.value_of::<i32>("audoff").unwrap(),
            cycle_limit: args.value_of::<usize>("cycle_limit").unwrap(),
            dbg_out: args.value_of::<bool>("debug").unwrap(),
        } ),
        Err(err) => {
            println!("{}", err);
            if !done_help {
                println!("{}", args.full_usage());
            }
            Err("Error parsing arguments".to_string())
        }
    }
}

