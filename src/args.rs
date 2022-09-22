
use std::env;
use getopts::Occur;
use args::Args;


pub fn getargs() -> Result<Args, String> {
    let mut args = Args::new("ti-audvid-convert", "Comverts a given video and transforms it into an application to be played back on a TI-83+SE or TI-84+(SE) calculator");
    
    args.option("i", "input", "Source video file", "INP", Occur::Req, None);
    args.option("o", "out", "Output application name (max 8 characters)", "OUT", Occur::Req, None);
    args.option("d", "duration", "How many frames (20fps) to convert from the video, omit for entire video", "DUR", Occur::Optional, Some("0".to_string()));
    args.option("s", "start", "Which frame (20fps) to start on, default first frame", "ST", Occur::Optional, Some("0".to_string()));
    args.option("f", "fps", "Manually supply the fps instead of checking the video", "FPS", Occur::Optional, Some("0".to_string()));

    args.flag("m", "mute", "Flag - shuts me up");
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
    let v: Vec<String> = env::args().collect();
    for s in v {
        if s.eq("-h") || s.eq("--help") {
            println!("{}", args.full_usage());
        }
    }
    
    match args.parse_from_cli() {
        Ok(()) => Ok(args),
        Err(err) => {
            println!("{}", err);
            Err("Error parsing arguments".to_string())
        }
    }
}

