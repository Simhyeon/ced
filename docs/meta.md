### TODO

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

* [ ] Quite a handful of operations are blindly assuming that indexing doesn't fail
Check all codes that indexs vector of hashmap and set appropriate error messages
	- Limiter opation
	- etc..

### DONE

* [x] Structure
* [x] Complete command line interface
* [x] Implement command history with memento pattern for easier implementation.
Make it swappable so that command pattern can be used later.
* [ ] Test EditRow command throughly
* [x] Add-row raw value
