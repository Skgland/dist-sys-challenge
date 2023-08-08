Project for my Implementation for the Gossip Glomers Challenges by Fly.io
=========================================================================

[Gossip Glomers by Fly.io](https://fly.io/dist-sys/)

Note: The tests do not check that the efficiency targets e.g. for braodcast part d/e are met 
those need to be verified manually, as of writing they are not fullfilled.

# Run

- Install all pre-requisits
  Note: the tests use `bash` to start `maelstrom`
- Download maelstrom and extract it into the project directory. 
  Such that it can be run with `$PROJ_DIR/maelstrom/maelstrom` from a bash shell.
- Run `cargo test` or `cargo test --release`