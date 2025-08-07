# simply-plural-to-any-updater

Update your [Simply Plural](https://apparyllis.com/) system fronting status automatically to
* your [VRChat](https://hello.vrchat.com/) status message
* your website as HTML

## SimplyPlural to VRChat Status

When running locally as a VRChat-Updater, it'll check the fronting status
on SimplyPlural periodically and update the VRChat status to reflect the current fronts
e.g. `F: Alice, Bob, Claire`.

For this, simply [download the latest executable](https://github.com/GollyTicker/simply-plural-to-any-updater/releases/latest) and run it locally. It'll create an empty file and ask you to put in your SimplyPlural and VRChat credentials.
These credentials are necessary for it to do it's job. After writing the credentials,
run the executable again. It will first login into VRChat. You may need to provide
a 2FA code, if you hav configured one for your account. Then it'll automatically
update your status in VRChat priodically from SimplyPlural. The login is saved in a cookie,
so you won't need to input your 2FA code that often.

## SimplyPlural to Website

When running as a website `--webserver`, it serves an endpoint `/fronting`
and provides a HTML page with the current fronting status (from SimplyPlural)
as a well-rendered UI.

To run the webserver, simply define a `deploy.env` with the relevant variables and run `restart-services.sh`. It uses a docker compose setup. You can stop services via `stop-services.sh`.

## FAQ

**Why is my member name not shown correctly?**

VRChat has limitations on what it allows one to show in the VRChat Status message.
While most european letters and accents are supported, special things such as emojis are not.
Hence this tool removes them before forwarding them to VRChat. If you think something is being removed,
while it's actually possible in the VRChat status, then shortly contact me and let me know (or write an issue).

Furthermore, if a member has a name which cannot be represented at all, e.g. `💖⭐`, then you can define a new
custom field in your Simply Plural named `VRChat Status Name` and fill in a VRChat compatible name in that field,
e.g. `Sparkle Star`. This way you can keep on using the proper name in Simply Plural while also having
something readable in VRChat.

## Migrate from v1 to v2

There are a few breaking changes in how to run this program:
* `SERVE_API` is removed. If it was `true`, then instead invoke the program with `--webserver`. Otherwise don't use this argument.
* The program now opens a GUI by default. If you want to keep on using the console only (old behavior), invoke the program with `--no-gui`.

## For Developers

Follow [these steps to install tauri](https://tauri.app/start/prerequisites/) for the rust GUI for local development.

Build without tauri: `./release/cargo-build.sh`
Build with tauri: `./dev/tauri-build.sh`
Build with tauri with live server and hot replacement: `./dev/tauri-dev.sh`

Lint and Format: `./release/lint.sh`

The environment variables are documented in `defaults.env` and `vrcupdater.sample.env`.

All functionality is implemented using Rust and various libraries.

For developers, one can use `/dev/*.run.sh` for local quick running.

And run the files in `test` for testing. For the integration tests,
you'll need to export the `SPS_API_TOKEN` and `SPS_API_WRITE_TOKEN` of the plural system used for tests - 
as well as `VRCHAT_USERNAME`, `VRCHAT_PASSWORD` and `VRCHAT_COOKIE` of the VRC test user.

To create a release, simply push a corresponding tag - e.g. `v1.2.3`.

Use `--config <filepath>` to specify an alternate directory where the config is stored and retrieved from.

Check dependencies bloat via `cargo bloat --release --bin sp2any`.

Use the following prompt against the code agent to put it to work:
```
Ensure the project adheres to the coding guidelines.
```
or
```
Update the dependencies.
```

### Current Migration

Migrate from directory local .env files to storing a .json in the home directoy.
That can be manually edited as well as configured via the GUI.

* default behavior for default configuration remains same
  * config JSON file is created, if none exists
  * only the values, which the user explicitly set are written into the config file
  * all other values are fetched from online from the github.com defaults
* GUI fetches and displays values from JSON
* don't offer a way to migrate from old .env file. keep code simple there.
* update README.md

### TODO

* Add documentation about discord sync
* Make it such that discord/vrchat sync is enabled specifically and don't have to be both activated at the same time
* Rename 'VRChat Status Name' field to 'Clean Status Name' field
* Ask on Reddit and various discord servers for what features the users want
* make ./release/lint.sh into CI. also add check that generated config example.json is equal to comitted one.
