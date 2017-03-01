##twitch-dl

This is a rust port of my [old lua implementation](https://github.com/Candunc/old.twitch-dl) of twitch-dl. This port is _actually_ functional and removes the dependance on both a local lua interpreter and aria2c, replacing them both with a native Rust application.

There are still some bugs to work out, namely the panic on uploaded videos, however it is actually quite functional.

To run the app, cd to the directory and run:

    cargo run <twitch url> <quality>

Where quality is highest, lowest, or anything else specified in the dialog when running with no arguments.

When a release is made, I'll fix up a nice website and make actual documentation.