# Skrillax Universal Patcher

This patcher allows up- _and_ downgrading of Silkroad Online in conjunction with
the [Universal Patch Server](https://github.com/kumpelblase2/skrillax-universal-patch-server/).
Essentially, this patcher is just a helper tool to get the official client to select the right version to patch to.
It is not meant to be used as a tool to permanently patch the game to use a different gateway server.

> [!WARNING]
> Work in Progress
>
> This is still a work in progress. If you want to use it, you probably need to make adjustments directly for it to work
> at all. It is unlikely to work out of the box for you.

## Usage

This currently expects you to run the universal patch server locally, so make sure that is running. Then place the
client patcher executable inside the Silkroad Online directory. You can then run it from inside the Silkroad Online
directory and pass it the desired version as an argument[^1]. Wait for the launcher to patch and restart. Once the
client has restarted, the patching has completed. Note that when the launcher restarts, it will already connect back to
the original gateway server, which may cause it to patch again. It's advised you patch to the desired gateway first,
before running this tool.

## How it works

For an explanation as to how this works, please see the README of
the [universal patch server](https://github.com/kumpelblase2/skrillax-universal-patch-server/?tab=readme-ov-file#how-it-works).

[^1]: Note the version here is the internal version, not the one displayed in the client. If the client displays
`v1.594`, the internal version is just `594`.
