### Ced, a csv editor and library

Ths is a csv editor and a backend for other frontends.

Ced is not a fully featured editor, but more likely an "ed" for csv. Ced simply
prevents you from adding surplus column or invalid data type. 

That's it. No searching, filtering, no nothing.

### Binary usage

```bash
# Start ced
ced

# Get help
>> help

# Import file
>> import file_name.csv

# Append new row to last
>> add-row 
First Header = .. <USER_INPUT>
Second Header = .. <USER_INPUT>
```

### Yet to come
- Import with csv schema for easier limiter set
- Limiter support in built-in command line interface
- Pretty print with custom csv viewer
