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

When running as a website (`SERVE_API=true`), it serves an endpoint `/fronting`
and provides a HTML page with the current fronting status (from SimplyPlural)
as a well-rendered UI.

To run the webserver, simply define a `deploy.env` with the relevant variables and run `restart-services.sh`. It uses a docker compose setup. You can stop services via `stop-services.sh`.

## For Developers

The environment variables are documented in `defaults.env` and `vrcupdater.sample.env`.

All functionality is implemented using Rust and various libraries.

For developers, one can use `dev.*.run.sh` for local quick running.

To create a release, simply push a corresponding tag - e.g. `v1.2.3`.

### Next Steps
* support accents such as š in member names in the VRChat status
* check the "VRChat Status Name" field for members and use that instead, if provided. That one will be VRChat Status compatible
