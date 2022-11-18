# Calc-Video-Generator




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
The rabbitsign folder must either exist in the current working directory or must exist in the path environment variable

If you intend to make changes to the z80 source code you will additionally need to install spasm-ng


# Quick Use

To generate a video app, use the following command in the command prompt:

Linux: `tiVidConvert -v <Video File> -o <Output File Name> -n <App Name on Calculator>`
Windows: `tiVidConvert.exe -v <Video File> -o <Output File Name> -n <App Name on Calculator>`


# Documentation



# License

This project is licenced under the MIT Licence, see LICENCE for more information
Rabbitsign is also provided alongside the Windows download and is licenced under the GNU General Public License



