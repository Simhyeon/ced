### Ced, a csv editor and library

Ths is a csv editor and a backend for other frontends.

Ced is not a fully featured editor, but more likely an "ed" for csv. Ced simply
prevents you from adding surplus column or invalid data type. 

### Install

```bash
cargo install ced --features cli
```

### Binary usage

**Ced option**

```bash
# Print version
ced --version
# Print help
ced --help

# Import schema and import data file.
# Execute a given command without opening an interactive shell
ced --schema schema.csv data.csv --command 'add-row 1 100,20;write'
```

**Ced shell command**

```bash
# Type help in prompt or give --help flag for detailed usage.

# Start ced
# Optionaly with initial import
ced
ced file.csv

# Get help
>> help

# Import file
>> import file_name.csv

# Import schema file
>> schema ced_schema.csv

# Print csv data optionally with viewer command
# Set CED_VIEWER for default print viewer
# Custom viwer will work for windows platform
>> print
>> print tidy-viwer

# Append new row to last
# Type comma to exit add loop
>> add-row 
First Header = .. <USER_INPUT>
Second Header = .. <USER_INPUT>

# Edit a given row
>> edit-row <ROW_NUMBER>

# Set limiter for a column with interactive shell
>> limit

# Import schema file with force update
>> schema file_name true

# Export to file
>> export file_name.csv

# Overwrite to the imported file
>> write

# Undo previous operation
# History capacity is 16 by default
# You can override it with CED_HISTORY_CAPACITY
>> undo

# Redo previous undo
>> redo
```
