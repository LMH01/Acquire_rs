## Acquire_rs

This is my try at writing the board game Acquire in rust to be played fia the command line.

### Getting started

- Download and unzip the `.zip` file specific for your os
- Open the shell in the extracted folder
- Launch the game by typing
	- `./acquire_rs.exe -p NUMBER_OF_PLAYERS` on Windows
	- `./acquire_rs -p NUMBER_OF_PLAYERS` in Linux

### Examples

`acquire_rs --lan-server -p 3 --name LMH01`

This will start a server on port 11511 on your local machine. The name of the local player is set to `LMH01` and the number of players is set to 3.

`acquire_rs --lan-client --name LMH01 --ip 192.168.178.1:11511`

This will launch a client instance on your computer that tries to connect to the server at `192.168.178.1:11511`. The player name is set to `LMH01`.

### Features

- Colored terminal output
- The game can be played in multiplayer fia lan, even cross platform 
- All rules from the original game have been implemented in this project, except for the special rules when only two players play
- Command line arguments powered by clap, type `acquire_rs --help` to view a list of all available commands
