# Calc-Video-Generator

A utility for generating videos to play on Ti-83+SE, Ti-84+, and Ti-84+SE calculators.
Converts given video files into application files to transfer to said calculators


# Download

### [Downloads](../../releases)
Note: Currently only a Windows build is available to download.
For Linux and other distributions follow to build instructions below.
If you're on Windows you can just download and skip to the quick use instructions.


# Building / Installation

Run `compile.sh` if building for Linux, `compile.bat` if building for Windows. The final executable will
be copied to the root project directory

Additionally, Rabbitsign must be installed for digital signing of the app.
Run `sudo apt-get install rabbitsign` or whatever instalation method is applicable to your system.
Rabbitsign must exist in the path environment variable or the current working directory

If you intend to make changes to the z80 source code you will additionally need to install spasm-ng


# Use Instructions

To generate a video app, use the following command in the command prompt:

Linux: `tiVidConvert -v <Video File> -o <Output File Name> -n <App Name on Calculator>`

Windows: `tiVidConvert.exe -v <Video File> -o <Output File Name> -n <App Name on Calculator>`

This should be sufficient for many videos. Make sure your videos aren't too long, as the calculators have limited
capacity in the archive for storage. The 83+SE and 84+SE have 94 app pages and can hold ~1 minute of video.
The 84+ non-se only has 30 app pages and holds about 20 seconds of video. Exactly how much you can fit depends on the
particular video being encoded and how much it can be compressed. The program will tell you how many pages

If your video is too long there are several things you can try. Running with `-d DUR` will encode only the first DUR calculator
frames, and `-s START` will change which calculator frame to begin encoding at.
By default video playback on the calculator runs at 20 FPS. If the video is very close to fitting, you can use `-p FPS` to lower
the playback framerate to make it take up less space. This option should stay close to 20 or else the video may start looking/sounding weird.

If the video seems desynced from the audio you can use `-a AUDOFF` to offset the audio playback
(greater value means audio begins playing sooner)

A list of all command-line arguments can be seen by running `tiVidConvert` with no options


# License

This project is licenced under the MIT Licence, see LICENCE for more information.

Rabbitsign is also provided alongside the Windows download and is licenced under the GNU General Public License.


# Bugs

There's bound to be a few I didn't catch. If you find any, either email me at ?????????? or ????????????

When reporting a bug please include as many of the following as applicable:
- A description of the bug
- The given error
- Video being converted
- Command-line options used
- Output applictaion file
- Any additional information which may be necesary to replicate the bug



