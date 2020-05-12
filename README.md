# rs_file_system_worker
File system worker

## Examples

### List a directory
This example will list the current directory.
```
RUST_LOG=info SOURCE_ORDERS=examples/list_directory.json cargo run
```

### Copy files
Create a directory named `tmp`

```
mkdir tmp
```
then run the example `copy_file.json`
```
RUST_LOG=info SOURCE_ORDERS=examples/copy_file.json cargo run
```

### Remove a directory
Remark: Create a `tmp` directory before run this example.

```
RUST_LOG=info SOURCE_ORDERS=examples/remove_directory.json cargo run
```
