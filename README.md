<p align="center"><img src="/resources/Design%205.PNG" alt="MOZAIC"/></p>

MOZAIC is the **M**assive **O**nline **Z**eus **A**rtificial **I**ntelligence **C**ompetition platform.
It aims to provide a flexible platform to host your very own AI competition. Just plug your game, and off you go!

Eventually MOZAIC should be very modular, so that you can provide a custom-tailored experience for your competitors, without having to worry about the heavy lifting.

# Features
For now MOZAIC is still in its early stages, but at Zeus WPI we're already getting it ready to support our own AI competition. As the platfrom matures, our specific usecase will be factored out and more features will be added.

Current and planned features:
 - [ ] Generic over game rules
   - [ ] implemented in Rust
   - [ ] ...
 - [ ] Game logging
 - [ ] Visualizers for your game
 - [ ] Uploading bots
    - [ ] Python or something
    - [ ] Any language, really
 - [ ] Fancy website
 - [ ] Network play
 - [ ] Flexible and performant game server
 - [ ] Friendly electron client
     - [ ] handles network play
     - [ ] "bot management"
  - [ ] Ranking bots

# Setup

## Botdriver

1. Install rust and cargo (take look [here](https://rustup.rs/) if you're using an older software repository such as Ubuntu apt)

- Rust >= 1.18.0
- Cargo >= 0.16.0

2. Try to run the botrunner with `cargo run` in the `src\botdriver` directory (it will fail to play a match).
3. Write a config for a match, or use the example in `src\games\planetwars\config_examples\stub.config.json`.
4. Run the botrunner again with `cargo run ..\games\planetwars\config_examples\stub.config.json` (still in the `src\botdriver` directory).
5. It should have generated a log-file `gamelog.json` (or whatever you specified in the config).
6. If it did, it works, check setup below for the gameserver.

## Blockly Game Server

1. Install Node v8
2. `npm install` in `src/games/planetwars/visualizer` en `src/games/planetwars/blockly` en `src/games/planetwars/gameserver`
3. `npm install webpack` or `(sudo) npm install webpack -g`
4. Run `webpack` in `src/games/planetwars/client`
5. Run `cargo build --release` in `src/botdriver/`
6. Symlink the botdriver from in the `src/games/planetwars/gameserver/` with `ln -s ../../../botdriver/target/release/mozaic_bot_driver bot_driver`
7. Run `node server.js` in `src/games/planetwars/gameserver`
8. The magic happens at `localhost:3000`
9. You're ready to see the game.

We have no idea if we support Windows, we probably do.

# Contact
Have any questions, comments, want to contribute or are even remotely interested in this project, please get in touch!
You can reach us by [e-mail](mailto:bestuur@zeus.ugent.be), [Facebook](https://www.facebook.com/zeus.wpi), or any other way you prefer listed [here](https://zeus.ugent.be/about/).
