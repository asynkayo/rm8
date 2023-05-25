Here are some hints to get rm8 running on various devices.

## Windows

It should work, as it is, but if you get an error about `VCRUNTIME140` install [this](https://www.microsoft.com/en-us/download/details.aspx?id=52685).

If the latency is too high, try adding `-smp 512` param.

## Mac

- [Install Homebrew](https://brew.sh/), a great general tool for managing libraries and stuff
- Open a terminal and run `brew install sdl2`
- Get latest [release](https://github.com/konsumer/rm8/releases) for your mac (M1/x86_64)
- open it and go to settings/privacy and ok it running an unsigned app
- If sound is not working, then the loopback is failing (whcih sometimes happens on mac, for some reason) but you can open "Quicktime Player" and choose "New Audio Recording" in "File" menu, then select "M8" and turn up the monitor slider. I'd like to figure out why it doesn't work like other platforms, but for now this will work (similar to m8c)

## Linux

- Get latest [release](https://github.com/konsumer/rm8/releases) for your arch (aarch64/x86_64)
- Install SDL2 (on ubuntu/debian/etc: `apt install libsdl2-2.0-0`)
- Make sure you have read/write permissions on the device (`./rm8 -list` with it plugged in to see devices) Generally just adding yourself to the group that owns the device will do it.


## Anbernic RG353P / RG353V

| RG353P      | RG353V |
| ----------- | ----------- |
| <img src="https://user-images.githubusercontent.com/83857/209609257-1da08aca-d8fa-48cc-98ed-e3e54d89136e.jpeg " width=450px>     |  <img src="https://user-images.githubusercontent.com/4543448/230634284-e7a50736-167e-4dba-9113-20467937c82b.jpg" width=450px>       |

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

# For RG353P
cp rm8-RG353P.json rm8.json

# For RG353V
cp rm8-RG353V.json rm8.json 

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

- Copy URL for [latest rm8-linux-x86_64.zip release](https://github.com/konsumer/rm8/releases)

On your computer `ssh deck@IP_ADDRESS` ([setup ssh](https://shendrick.net/Gaming/2022/05/30/sshonsteamdeck.html)), or you can use desktop-mode terminal:

```
cd ~
mkdir M8
cd M8

# ZIP_URL come from releases: x86-64
wget ZIP_URL -O r.zip
unzip r.zip
rm r.zip

# add main user to group that owns M8
sudo usermod -a -G uucp deck
```

- in Desktop-mode, open steam, add non-steam game for `/home/deck/M8/rm8` and set working-dir to `/home/deck/M8`
- In controller-configuration, choose the `rm8` community-profile I (konsumer) made.
- Reboot
