# Instructions for AI Coding Agents

First read `README.md` to get an overview of what the project does.

Only do the tasks described in the following sections if explicitly requested to.

## Coding Guidelines
* Rust import statemnts should be one crate per statement. Importing multiple objects from the same create should be done in the same statement.
  * Good: `use anyhow::{anyhow, Error, Result}`
  * Bad: the above imports on separate lines/statements for each imported object
* Rust import statements should use separate lines for imports from different modules originating from this project.
* After you made all changes, run a final `./release/lint.sh` and summarise the changes to make.
  * Warnings or errors are only allowed for files which have a `WORK-IN-PROGRESS` marker at the top.


## Update Dependencies
Run:
* `cargo update`
* download `npm-check-updates` and run it over the npm packages in `frontend`
Then commit the changes and request to push them onto a new branch `dependencies/update-YYYY-MM-DD--hh-mm` where the date format variables are accordingly used.
