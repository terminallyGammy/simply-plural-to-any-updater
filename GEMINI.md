# Instructions for AI Coding Agents

First read `README.md` to get an overview of what the project does.

## Coding Guidelines
* Rust import statemnts should be one crate per statement. Importing multiple objects from the same create should be done in the same statement.
  * Good: `use anyhow::{anyhow, Error, Result}`
  * Bad: the above imports on separate lines/statements for each imported object
* Rust import statements should use separate lines for imports from different modules originating from this project.
* After making changes, run a final `./release/lint.sh` and summarise the changes to make.
  * There should be no warnings or errors in files which don't have a `WORK-IN-PROGRESS` marker at the top.
