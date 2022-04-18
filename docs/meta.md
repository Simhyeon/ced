### TODO

- In built cli's interface should be simple and easy but also lightweight and fast.
- No bloat: dependencies are better when smallest

* [x] Make a single pass interface for cli
	- With --no-confirm option
	- Else always trigger confirm screen if use is satisfied with result
	* [x] For this I need a complete argument parser.
		- This is almost complete
		- Test if cli arguments are passed to parser properly
		- Make a branch where parsed flags are properly processed.

* [x] Print cell + print low's print mode
* [ ] Makefile for help is outrageous... but not the priority
* [x] Drop thiserror

* [x] Edit-rows option, maybe?
* [x] Help with command arguments
* [x] Add "CED\_VIEWER" variant
* [x] Edit-row interactive
	- With default value validation

* [x] Enable limit one-line
* [x] Fixed limit inconsistency
* [x] Made add-column more versatile

-> Sanity

* [ ] Library usage ergonomic binding

-> Hard

* [ ] Get + Set selection api ( Consider if this is necessary, not mandate but eaiser for end users )
* [ ] Proper undo ( No memento if possible )

### NOT TODO

Some great ideas but not suitable for this in-built cli interface is placed here.

- Multi csv virtual table.
- Selection 
- Option of always print

### DONE

* [x] Structure
* [x] Complete command line interface
* [x] Implement command history with memento pattern for easier implementation.
Make it swappable so that command pattern can be used later.
* [x] Add-row raw value

* [x] Column index is bugged
* [x] Alert user that command didn't succeed
* [x] Should not be able to add duplicate column
* [x] Enable setting limiter in cli
* [x] Print-column to print data about column
* [x] Support complicated viewer method -> Add environmental variant

* [x] Add schema support for easy csv import
	- Read schema
	- Create schema
	- Export schema

	- Turns out that official csv schema is very complicated... and I don't
	think I need to implement in near future.
	- Add simple csv sheet which describes csv value.

