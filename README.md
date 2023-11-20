# Obligatory Chat App

## Preamble

I used SQLite when I wrote the initial code. Install `sqlx-cli` and follow the docs via [this documentation](https://lib.rs/crates/sqlx-cli) and create your to get started.

My sqlite3 DB resides in the `./data` directory, but it can live anywhere, as long as the DB in your `.env` points to the DB file on your local HDD.

## .env file

Create a `.env` file in the root directory and copy the contents of `.env.example` into it (replacing the placeholder values with real ones).

## Running the app

1. `cargo update`
2. `cargo run`

Should be good to go after that.

## Notes

This app is by no means finished. A half-assed session + cookie layer has been implemented, but still needs proper session expiration checks. There are a couple of TERA templates that render the `/chat`, '/login`, and `/sign-up` pages. Visiting the index page (`/`) should check your session cookie and redirect you to `/login` or`/chat`, depending on its presence/validitiy. The websocket layer establishes a connection, but actual back n' forth chat functionality is yet to be implemented.
