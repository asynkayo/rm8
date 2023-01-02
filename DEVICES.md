Here are some hints to get rm8 running on various devices.

## Windows

It should work, as it is, but if you get an error about `VCRUNTIME140` install [this](https://www.microsoft.com/en-us/download/details.aspx?id=52685).

## Mac

- [Install Homebrew](https://brew.sh/)
- Open a terminal and run `brew install sdl2`
- Get latest [release](https://github.com/konsumer/rm8/releases) for your mac (M1/x86_64)
- open it and go to settings/privacy and ok it running an unsigned app
- Currently, sound is not working on Mac, but you can open "Quicktime Player" and choose "New Audio Recording" in "File" menu, then select "M8" and turn up the monitor slider. I'd like to figure out why it doesn't work like other platforms, but for now this will work (similar to m8c)

## Linux

- Install SDL2 (on ubuntu/debian/etc: `apt install libsdl2-2.0-0`)
- Make sure you have read/write permissions on the device (`./rm8 -list` with it plugged in to see devices) Generally just adding yourself to the group that owns the device will do it.


## Anbernic RG353P

![RG353P](https://user-images.githubusercontent.com/83857/209609257-1da08aca-d8fa-48cc-98ed-e3e54d89136e.jpeg)

- Install [JELOS](https://github.com/JustEnoughLinuxOS/distribution)
- In network-settings enable wifi & ssh
- Copy URL for [latest rm8-linux-aarch64.zip release](https://github.com/konsumer/rm8/releases)

On your computer `ssh root@IP_ADDRESS` (password is in settings, chnages on every boot):

```
cd ~/roms/ports
mkdir M8
cd M8

# ZIP_URL come from releases: aarch64
wget ZIP_URL -O r.zip
unzip r.zip
rm r.zip

cp rm8-RG353P.json rm8.json

# make it look like script below
nano ../M8.sh
```

*~/roms/ports/M8.sh*
```sh
#!/bin/bash

DIR="$( dirname "${BASH_SOURCE[0]}"; )"
DIR="$( realpath "$DIR" )"

cd "${DIR}/M8"
./rm8
```

Input is mapped so SELECT exits, and START is CONFIG. Dpad and analog works, and face-buttons are for editing (similar to real M8.)


## Steamdeck

![steamdeck](https://user-images.githubusercontent.com/83857/209611069-7cf42ce3-7690-42ba-8d52-d511f68faf95.jpeg)

- [setup ssh](https://shendrick.net/Gaming/2022/05/30/sshonsteamdeck.html)
- Copy URL for [latest rm8-linux-x86_64.zip release](https://github.com/konsumer/rm8/releases)

On your computer `ssh deck@IP_ADDRESS`, or you can use desktop-mode terminal:

```
cd ~
mkdir M8
cd M8

# ZIP_URL come from releases: x86-64
wget ZIP_URL -O r.zip
unzip r.zip
rm r.zip
```

- in Desktop-mode, open steam add non-steam game for `/home/deck/M8/rm8` and set working-dir to `/home/deck/M8`
- In controller-configuration, choose the `rm8` community-profile I (konsumer) made.
