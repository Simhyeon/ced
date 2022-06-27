# 0.2.2

- BugFix : Limit command loop was not interruptable
- BugFix : Limit command one-liner value didn't respect quote rules
- BugFix : Limit command loop input trimmed all whitespaces
- BugFix : Undo and redo mechanics were inconsistent
- Ergono : Line number prompt for edit row multiple
- Ergono : Limit command show columns before prompt
- Ergono : Print-column now allows no argument case
- Featur : New command history

# 0.2.1

- Hotfix : Ced panicked when given invalid command type

### Known issues

- Loop variants are not properly parsing quoted values
	- Add row
	- Edit row
	- Limit

# 0.2.0

- Featur : New command import-raw for raw editing mode
- Change : Updated dcsv version with many internal breaking changes
- Bugfix : Strange read\_input in windows platform.
- Ergono : Write now panics when there is no source file.
- Ergono : More documentations
- BugFix : Previously import always added but never shrinked cleaned csv data

# 0.1.7

- BugFix : Print row only worked when viewer was set.
- BugFix : Header's 'H' character was not properly lined
- BugFix : Rename column didn't make sure column name was valid.
- BugFix : Limiter's force argument was set opposite.
- Ergono : Binary help now prints shorter version without shell commands.
- Ergono : New import argument to read old MacOS file (CR Line ending file)
- Ergono : Disable loop variant when executed directly from binary
- Ergono : Changed tokenizing behaviour so that user can input whitespaces in element with quotes.

