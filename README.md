# RM8

Remote display for the [Dirtywave M8](https://dirtywave.com/)

I first tried M8WebDisplay then discovered m8c and decided to use it as a starting point for my own version in rust..

# Config

You can get device-specific tips in [DEVICES](https://github.com/konsumer/rm8/blob/master/DEVICES.md).


[SDL scancodes](https://github.com/libsdl-org/SDL/blob/main/include/SDL_scancode.h)

[SDL keycodes](https://github.com/libsdl-org/SDL/blob/main/include/SDL_keycode.h)

`Alt + Enter` will toggle fullscreen.

`Alt + R key` will reset the display

`Alt + Shift + R key` will do a full reset of the display (disconnect + enable and reset display)

`Alt + C` will enter config mode. THink of this as "extreme dev secret everything-broke menu". Many options will crash rm8.

`Escape` will either quit the application or fullscreen mode or config mode or key remapping mode.

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

## Keyjazz keymapping

The following keys are used in `keyjazz` mode to send notes to the M8:

- Higher octave:  Q 2 W 3 E R 5 T 6 Y 7 U I 9 O 0 P
- Current octave: Z S X D C V G B H N J M , L . ;

If you are using an `azerty` or `dvorak` keyboard layout, you may want to change the keys.

For that, open or generate the config file and look for the `"keyjazz"` section.

The numbers on the right are the index of the note.

NOTE: If the keys do not overlap with `keyjazz` keys, then `keyjazz` can be left ON.

## Usage

Run `rm8 -help` 	to display the help screen.

Run `rm8 -list` 	to list available M8 devices.

Run `rm8 -dev DEVICE` 	to connect to the specified M8 device.

Run `rm8 -cap "M8 Analog Stereo (2)"` 	to connect the given capture device to the default playback device.

Run `rm8 -wc` 		to print the default config to the standard output.

Run `rm8 -wc <FILE>` 	to write the config to the given file.

Run `rm8 -rc <FILE>` 	to load the config from `FILE`.

NOTE: The default config file name is `rm8.json`.

# Audio

`rm8` can now directly output the audio from your M8 !
You can either run `rm8` and let it open the first M8 capture device it found or you can use the `-cap` command line argument to specify which M8 you want to use.
For now there is no support for this feature in the in-app config system.

# Config Mode

By pressing `Alt + C` you will enter config mode.

In this mode, you can redefine most params of the application.

The parameters are split in 8 pages.

Pressing `Edit` and `Option` on a control will reset it to its default value.

Navigate using `Shift` and `ARROWs` just like on the M8.

Press `Escape` at any time to exit config mode. (Press twice if your were in remapping mode).

## Application config

On this page you will be able to configure:

- Fullscreen (effective after a restart but you can use `Alt + Enter` to toggle fullscreen).
- Zoom level
- Font options (see Alternate Fonts)
- Key sensibility
- Show FPS
- FPS (select desired number of FPS)
- Reconnect (when using only one M8 device, try to reconnect in case the connection is lost, the default behavior is to quit)
- Device (when using multiple M8 devices, switch between them with this setting)

Press `RESET` to restore the application settings to their last saved state.

Press `SAVE` to save the application settings to the config file.

## Theme config

On this page you will be able to configure the colors of the application:

- Text:Default
- Text:Value
- Text:Title
- Text:Info
- Cursor
- Screen Background
- Velocity indicator (Background and Foreground)
- Octave indicator (Background and Foreground)

Press `RESET` to restore the theme settings to their last saved state.

Press `SAVE` to save the theme settings to the config file.

## M8 Keys

On this page you will be able to map the keys to control your M8:

- UP
- DOWN
- LEFT
- RIGHT
- EDIT
- OPTION
- SHIFT
- PLAY

Press `REMAP` button to enter remap mode and redefine the keys.

Press `Escape` to exit remapping mode.

Press `RESET` to restore the M8 key settings to their last saved state.

Press `SAVE` to save the M8 key settings to the config file.

## RM8 Keys

On this page you will be able to map the keys to control the application:

- KEYJAZZ (toggle keyjazz mode)
- VELOCITY-
- VELOCITY+
- OCTAVE-
- OCTAVE+

Press `REMAP` button to enter remap mode and redefine the keys.

Press `Escape` to exit remapping mode.

Press `RESET` to restore the RM8 key settings to their last saved state.

Press `SAVE` to save the RM8 key settings to the config file.

## Joysticks

On the main config page you will be able to select your joystick.

If no joysticks are connected, you will only see this information:

- N.JOYSTICKS 0

If you have at least one joystick connected, you will see this information:

- N.JOYSTICKS N (for as many joysticks you have connected)
- SEL.ID      0 (use this control to select your joystick)
- NAME          (the name of the selected joystick)
- GUID -----    (this is the unique identifier of your joystick, it will be used in the config file).
- N.AXES      A (number of axes)
- N.BUTTONS   B (number of buttons)
- N.HATS      H (number of hats)

Press `RESET` to restore ALL the settings of your joystick to their last saved state.

Press `SAVE` to save the ALL the settings of your joystic to the config file.

### Axes

On this page you will be able to configure the `axes` of your joystick.

Currently, only 6 axes are supported.

For each axis, you will be able to associate 2 commands (for negative and positive).

You will also be able to configure the axis sensibility in order to avoid spurious triggers.

Press `RESET` to restore the Axes settings to their last saved state.

Press `SAVE` to save the Axes settings to the config file.

### Buttons

On this page you will be able to configure the `buttons` of your joystick.

Currently, only 20 buttons are supported.
For each button, you will be able to associate 1 command.

Press `RESET` to restore the Buttons settings to their last saved state.

Press `SAVE` to save the Buttons settings to the config file.

### Hats

On this page you will be able to configure the `hats` of your joystick.

Currently, only one hat is supported.

For each state of the hat, you will be able to associate 1 command.

The states are:

- UP
- DOWN
- LEFT
- RIGHT
- UP LEFT
- UP RIGHT
- DOWN LEFT
- DOWN RIGHT

Press `RESET` to restore the Hat settings to their last saved state.

Press `SAVE` to save the Hat settings to the config file.

### NOTE

The code for handling joysticks may be a bit buggy as I do not have enough experience in dealing with these devices.

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

## Alternate fonts

I modified the bitmap rendered font to support both alternate zeros from the newer firmwares.

However, as I do not yet have a real M8, I don't know if there are differences with the font on newer firmwares.

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

