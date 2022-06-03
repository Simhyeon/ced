### Ced, a csv editor and library

Ths is a csv editor and a backend for other frontends.

Ced is not a fully featured editor, but more likely an "ed" for csv. Ced simply
prevents you from adding surplus column or invalid data type. 

[Changes](./docs/change.md)

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

# Execute script
# argument with .ced extension will be interpretted as execution script
# In this case, loop variants are restricted
ced script.ced

# Shell commands
# Get help
>> help

# Import a file
>> import file_name.csv

# Import a schema file. Second argument determines overriding.
>> schema file_name true

# Print csv data optionally with a viewer command
# Set CED_VIEWER to set default print viewer
>> print
>> print tidy-viwer

# Append a new row to last
# Type a comma to exit loop
>> add-row 
First Header = .. <USER_INPUT>
Second Header = .. <USER_INPUT>

# Edit a given row
>> edit-row <ROW_NUMBER>

# Set a limiter for a column with interactive shell
>> limit

# Export to a file
>> export file_name.csv

# Overwrite to a source file
>> write

# Undo a previous operation
# History capacity is 16 by default
# You can override it with CED_HISTORY_CAPACITY
>> undo

# Redo a previous undo
>> redo
```

### Note

Currently CR(Old Mac OS line ending) format is not supported. CRLF and LF
works. This might change in future releases.
