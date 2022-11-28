# TI-Calc-Video-Generator

A command-line utility for generating videos to play on Ti-83+SE, Ti-84+, and Ti-84+SE calculators.

Converts a given video file into an application file to transfer to the calcuators.
Videos will play at 20 frames per second, with 4-level greyscale and 10.24 KHz audio playback from the link port
([Demonstration](https://www.youtube.com/watch?v=JiZ8KPonHsw))


# Download

### [Downloads](../../releases)
Note: Currently only a Windows build is available to download.
For Linux and other distributions follow the build instructions below.

ffmpeg must still be installed (see below)


# Prerequisites

This program uses ffmpeg to extract the audio and video frames.

Windows: [follow these steps for downloading ffmpeg for windows](https://www.geeksforgeeks.org/how-to-install-ffmpeg-on-windows/)

Linux: run `sudo apt-get install ffmpeg` or whatever instalation method is applicable to your system.

Windoes users may need to restart their computer.


# Building / Installation

Run `compile.sh` if building for Linux, `compile.bat` if building for Windows. The final executable will
be copied to the root project directory

Additionally, Rabbitsign must be installed for digital signing of the app.
Run `sudo apt-get install rabbitsign` or whatever instalation method is applicable to your system.
Rabbitsign must exist in the path environment variable or the current working directory

If you intend to make changes to the z80 source code you will additionally need to install spasm-ng


# Conversion Instructions

To generate a video app, use the following command in the command prompt:

Linux: `tiVidConvert -v <Video File> -o <Output File Name> -n <App Name on Calculator>`

Windows: `tiVidConvert.exe -v <Video File> -o <Output File Name> -n <App Name on Calculator>`

This should be sufficient for many videos. Make sure your videos aren't too long, as the calculators have limited
capacity in the archive for storage. The 83+SE and 84+SE have 94 app pages and can hold ~1 minute of video.
The 84+ non-se only has 30 app pages and holds about 20 seconds of video. Exactly how much you can fit depends on the
particular video being encoded and how much it can be compressed. The program will tell you how many pages the converted
video is taking up.

If your video is too long there are several things you can try. Running with `-s START` and `-d DURRATION` will let you encode only a
certain section of the video. The start position and durration are measured in calculator frames (default 20 fps).

If the section of video is very close to fitting, you can also use `-p FPS` to lower the playback framerate to make it take up less space.
This option should stay close to 20 or else the video may start looking/sounding weird.

If the video seems desynced from the audio you can use `-a AUDOFF` to offset the audio playback
(greater value means audio begins playing sooner)

A list of all command-line arguments can be seen by running `tiVidConvert` with no options


# Calculator Playback Instructions

The program will output an application file for use on your calculator. Use either [TI Connect](https://education.ti.com/en/products/computer-software/ti-connect-sw)
or [TiLP](http://lpg.ticalc.org/prj_tilp/) to transfer to your calculator.
If your app does not fit either free up space on your calcuator or try and reduce the space taken by the video (see above).

Once on the calculator you will be able to see your video in the APPS menu. After selecting the application you will be asked to tune the display settings,
then after pressing ENTER your video will begin playing.

Audio plays from the I/O port. Either plug in headphones/speekers with a 2.55mm connection, or more likely use a [3.55mm to 2.55mm addapter](https://www.amazon.com/Vention-Adapter-Converter-Headphone-Earphone/dp/B07NSVBVQN/ref=sr_1_6?keywords=3.55%2Bto%2B2.55%2Badapter&qid=1669496659&s=electronics&sprefix=2.55%2Celectronics%2C119&sr=1-6&th=1)


# License

This project is licenced under the MIT Licence, see LICENCE for more information.

Rabbitsign is also provided alongside the Windows download and is licenced under the GNU General Public License.


# Bugs

There's bound to be a few I didn't catch. If you find any, either email me at crazyfox2a@gmail.com or open an issue on the github page

When reporting a bug please include as many of the following as applicable:
- A description of the bug
- The given error
- Video being converted
- Command-line options used
- Output applictaion file
- Any additional information which may be necesary to replicate the bug

