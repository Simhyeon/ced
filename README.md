### Ced, a csv editor and library

Ths is a csv editor and a backend for other frontends.

Ced is not a fully featured editor, but more likely an "ed" for csv. Ced simply
prevents you from adding surplus column or invalid data type. 

That's it. No searching, filtering, no nothing.

### Install

```bash
cargo install ced --features cli
```

### Binary usage

```bash
# Start ced
ced

# Get help
>> help

# Import file
>> import file_name.csv

# Print csv data optionally with viewer command
# Custom viwer may not work windows platform
>> print
>> print tidy-viwer

# Append new row to last
>> add-row 
First Header = .. <USER_INPUT>
Second Header = .. <USER_INPUT>

# Export to file
>> export file_name.csv

# Overwrite to imported file
>> write <USE_CACHE: boolean>
```

### Yet to come
- Library usage and ergonomic binding
- Import with csv schema for easier limiter set
- Limiter support in built-in command line interface
