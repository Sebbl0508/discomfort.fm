# Discomfort.FM
Discord bot for listening to webradio. (But it can also be used for playing other audio URLs)

## Usage (no Docker)
Copy `.env.example` to `.env` and adjust the values:
- `DISCORD_TOKEN`: The discord bot token
- `SELF_DEAF`: The bot deafens itself so it doesn't hear conversations
    - *Not working yet*
- `MAX_VOLUME`: The maximum volume that can be set from discord
    - (set to something like `10000` for a fun time :D)
- `DATABASE_URL`: SQLite URI to where the database should be saved, if not set it will land in the local app data directory of your OS
    - In linux the default directory should be `$HOME/.local/share/discomfort-fm/data.db`
    - It is important to add `?mode=rwc` at the end of this string so that the database will be created, if it doesn't exist yet
- `PULISH_GLOBAL`: Set to `true` to (re-)register global application commands
    - This should be set `true` on initial run or after an update
    - (Maybe this will be done automatically in the future)

Build via `cargo build --release` then run the application at `target/release/discomfort-fm`.

## Usage (Docker)
Build your own image with the dockerfile provided or use the image `sebbl0508/discomfort-fm` (It does not exist yet :P).  
Then either set the environmental values above via docker or mount a `.env` file to `/app/.env`.  
You also should mount the database to somewhere, so it will not be reset when restarting/recreating the docker container.
I recommend setting `DATABASE_URL` to `sqlite:///data/data.db?mode=rwc` and then mounting `/data` via docker to somewhere.
