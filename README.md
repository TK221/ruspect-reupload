Re-upload of the game we created for the "Introduction to Rust-Development" course. 

Greetings to the co-author of this project: https://github.com/SimonWeissDHBW.

# ruspect
Ruspect is a little game to try out the features of bevy and learn Rust.
It is inspired by the Binding of Isaac.

# Requirements
`rust` - version 1.60

## Linux
Maybe you will need some packages for the game to work properly, try:

```
sudo apt install libasound2-dev
sudo apt install libudev-dev
```
For more information: [Bevy Cheatbook - Linux Desktop](https://bevy-cheatbook.github.io/platforms/linux.html)

# How to build
`cargo build -r` will build the target folder with the executable game.
You have to move assets folder into the target folder, so the game can work properly.  

## Run/Play the game
First navigate to the new folder `ruspect/target/release/`.

### Windows
Simply open the newly build file ruspect.exe

### Linux
Run the the file ruspect with the terminal.
```
./ruspect
```

Now you should be ready to start.  
Have fun.