# RM8

Remote display for the [Dirtywave M8](https://dirtywave.com/)

I first tried M8WebDisplay then discovered m8c and decided to use it as a starting point for my own version in rust..

# Config

[SDL scancodes](https://github.com/libsdl-org/SDL/blob/main/include/SDL_scancode.h)

`Alt + Enter` will toggle fullscreen.

`Shift + R key` will reset the display (enable and reset display)

`Alt + Shift + R key` will do a full reset of the display (disconnect + enable and reset display)

`Ctrl + r` will help you redefine all the mappings and save the result to `rm8.json`. Use `Esc` to cancel the process.

## Default key mapping
    UP        = UP           # M8's `UP` key
    DOWN      = DOWN         # M8's `DOWN` key
    LEFT      = LEFT         # M8's `LEFT` key
    RIGHT     = RIGHT        # M8's `RIGHT` key

    SHIFT     = LSHIFT       # M8's `SHIFT` key
    PLAY      = SPACE        # M8's `PLAY` key

    EDIT      = LCTRL        # M8's `EDIT` key
    OPTION    = LALT         # M8's `OPTION` key

    KEYJAZZ   = RETURN       # Enter keyjazz mode
    OCTAVE+   = RIGHTBRACKET # Increment octave
    OCTAVE-   = LEFTBRACKET  # Decrement octave
    VELOCITY+ = MINUS        # Increment velocity (use the keyboard's `Shift` key to go faster)
    VELOCITY- = EQUALS       # Decrement velocity (use the keyboard's `Shift` key to go faster)

NOTE: If the keys do not overlap with `keyjazz` keys, then `keyjazz` can be left ON

## Usage

Run `rm8 -help` 	to display the help screen.

Run `rm8 -list` 	to list available M8 devices.

Run `rm8 -dev DEVICE` 	to connect to the specified M8 device.

Run `rm8 -wc` 		to print the default config to the standard output.

Run `rm8 -wc <FILE>` 	to write the config to the given file.

Run `rm8 -rc <FILE>` 	to load the config from `FILE`.

NOTE: The default config file name is `rm8.json`.

# Build

This project uses [rust](https://rust-lang.org)

You need to install rust and then issue: `cargo build --release` in the project's directory.

The program can then be found in the directory `target/release`.

You can strip symbols from this binary using `strip rm8`.

You can also compress the binary to gain some extra bytes using `upx --best --lzma rm8`.

## Dependencies

This project uses:
- [SDL2](https://www.libsdl.org)
- [serialport](https://gitlab.com/susurrus/serialport-rs)
- [serde](https://serde.rs/)
- [ctrlc](https://github.com/Detegr/rust-ctrlc)

# Similar projects

[g0m8](https://github.com/turbolent/g0m8)

[m8c](https://github.com/laamaa/m8c)

[M8WebDisplay](https://github.com/derkyjadex/M8WebDisplay)

# Font

This project uses a bitmap render of the font stealth57.ttf by Trash80.

Original font available at https://fontstruct.com/fontstructions/show/413734/stealth57

Originally licensed under a Creative Commons Attribution Share Alike license, https://creativecommons.org/licenses/by-sa/3.0/

# Info from discord

[discord](https://discord.com/channels/709264126240620591/709264126664507393)

	M8 SLIP Serial Receive command list
	'S' - Theme Color command: 4 bytes. First byte is index (0 to 12), following 3 bytes is R, G, and B
	'C' - Joypad/Controller command: 1 byte. Represents all 8 keys in hardware pin order: LEFT|UP|DOWN|SELECT|START|RIGHT|OPT|EDIT
	'K' - Keyjazz note command: 1 or 2 bytes.
	       First byte is note, second is velocity, if note is zero stops note and does not expect a second byte.
	'D' - Disable command. Send this command when disconnecting from M8. No extra bytes following
	'E' - Enable display command: No extra bytes following
	'R' - Reset display command: No extra bytes following

	M8 SLIP Serial Send command list
	251 - Joypad key pressed state (hardware M8 only) - sends the keypress state as a single byte in hardware pin order:
	      LEFT|UP|DOWN|SELECT|START|RIGHT|OPT|EDIT
	252 - Draw oscilloscope waveform command:
	      zero bytes if off - uint8 r, uint8 g, uint8 b, followed by 320 byte value array containing the waveform
	253 - Draw character command: 12 bytes. char c, int16 x position, int16 y position,
	      uint8 r, uint8 g, uint8 b, uint8 r_background, uint8 g_background, uint8 b_background
	254 - Draw rectangle command: 12 bytes. int16 x position, int16 y position,
	      int16 width, int16 height, uint8 r, uint8 g, uint8 b

	so when connecting via serial, first send a E, then a R to get all the data back from M8
	its also important to end with a D

