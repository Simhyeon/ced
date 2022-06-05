### TODO

* [ ] Ced behaves very strange in windows paltform.
	* [ ] Especially after interactive input
* [ ] Write panic when ced didn't import any file
	- Add new message 
	- : No Source file to write. Use export instead : 
* [ ] Check limiter preset
* [ ] Create test.rs for easier testing

- In built cli's interface should be simple and easy but also lightweight and fast.
- No bloat: dependencies are better when smallest
* [ ] Library usage ergonomic binding

-> Hard

* [ ] Command read from stdin by default ( This is possible with simple read\_line solution )
* [ ] Automatic build tests
* [ ] Get + Set selection api ( Consider if this is necessary, not mandate but eaiser for end users )
* [ ] Make test script -> Ok... this is quite cumbersome
* [ ] Command based undo mechanic

### SEMI-DONE

* [-] Use different character when cell value is empty
* [-] Enable super pretty print with columns' current longest width value. ::
-> This might be helpful, but there are many good tools that can be better. To
follow unix philosophy I'd rather let it sink
* [-] Read file as trimmed option. -> Read value as trimmed form. :: -> This is
not hard... but is it really necessary for ced? Possibly not
* [-] Import as page name support? Hmm... This may not be necessary though
* [-] ~~Floating point type~~ : Not worth the hassle
* [x] Document CED\_HISTORY\_CAPACITY
* [x] Removed trailing comma in default print method
* [x] Support multi-page for virtual data
	- This is conceptually done but not thoroughly tested because current
	development is focuesed on simplest cil usage. This can be improved when
	proper tui implemenation is developed 
* [x] Command can literaly fail if no page is imported.
	- SOLUTION : Add empty page when starting a binary ( Command loop )
* [x] Added d-quoted comma support
	* [x] Also in edit cell method
	* [x] Should not strip completely because stripped value cannot be passed to other progams.
		* [x] Processor split method
		* [x] Utils strip method
		* [x] Command print method
* [x] Currently edit variants do nothing if data is empty which is confusing. Infer the emptiness to user.
* [x] Execute command
	* [x] Read command from file

* [x] Empty option flag is not an error and this is confusing

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

* [x] Value check on input rather than input on row based.
	- On loop, if value doesn't fit it should ask you to input again.
* [x] Exit from loop + comma should be escaped for sanity reason
* [x] BUG : Force update doesn't affect "empty value" although number type is set later.
* [x] BUG : Default value for schema was not applied when only default value was given.
* [x] Bug : Schema import was not working at all
* [x] Bug : Add-row didn't respect something -> What? something?

* [x] Performance improvement
	* [x] Remove clone calls if possible
	* [x] Separate set and edit so that edit can prevent allocation if possible.
		* [x] On number text you cannot escape with comma character because it
		is detected as type mismatch 
		* [x] Current implementation of while != None is completely broken fix this.
* [x] Added column numbers for printing
* [x] Print row command
* [x] Enable user to specify ced history capacity, so that user can also disable history
* [x] Organize mod structure -> Only expose what is necessary

* [x] Extract command to global feature
* [x] Windows compatible subprocess ( Ced Viewer )
* [x] Now arguments are order-insensitive
* [x] Disable log output
* [x] Disable loop variant for non cli build

* [x] Better print with numbers function
* [X] Improved default print formatting
* [x] Fixed empty row error
* [x] Print-row is broken... because it was only utilizing CED\_VIEWER and didn't print anything by default
* [x] Fix unlined header
* [x] Maybe help but shorter version? Also help but for binary only option maybe
* [x] Line ending import option ( CR )
* [x] Disable loop on execute argument by default.
* [x] Made argument parsing much more intuitive -> more like sh syntax
* [x] Rename-column should not allow number name : Dcsv problem.
* [x] Fixed wrong limiter behaviour
* [x] Bettered documentation

* [x] Limiter respects csv convention now
* [x] Limiter preset
	* [x] Include this in command\_loop

