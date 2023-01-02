Here are some hints to get rm8 running on various devices.

## Anbernic RG353P

![RG353P](https://user-images.githubusercontent.com/83857/209609257-1da08aca-d8fa-48cc-98ed-e3e54d89136e.jpeg)

- Install [JELOS](https://github.com/JustEnoughLinuxOS/distribution)
- In network-settings enable wifi & ssh
- Copy URL for [latest aarch64 release zip](https://github.com/konsumer/rm8/releases)

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
- Copy URL for [latest x86-64 release zip](https://github.com/konsumer/rm8/releases)

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
- In controller-configuration (under steam-button menu when it's running) choose the `rm8` community-profile I (konsumer) made.
