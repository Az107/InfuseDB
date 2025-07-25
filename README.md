# InfuseDB

**InfuseDB** is a minimalist embedded database written in Rust. It’s designed to be simple, interactive, and easy to use from the command line or embedded in an app. It provides basic persistence and document-style querying (similar to JSON).

---

## ✨ Key Features

- Store collections of documents.
- Command-line navigation and editing.
- Automatic persistence to disk via `commit`.
- Support for nested keys (`user.name`, `users.0.name`, etc.).
- Basic filtering with `where` and simple operators.
- Optional **TCP server mode** (`--features server`).

---

## 🚀 Basic CLI Usage

Running the binary with no arguments opens an **interactive REPL-style interface**:

```sh
$ infusedb
InfuseDB 0.1.0
default>
```

### General Commands (`no collection selected`)

```txt
help
    Show this help message.

exit
    Exit the application.

list
    List all available collections.

select <collection_name>
    Select a collection to work with.

new <collection_name>
    Create a new collection.

del_col <collection_name>
    Delete a collection. If a collection is currently selected, it will be deleted directly.

commit
    Save changes to the database.

Notes:
- You must select a collection to perform document operations.
- If no collection is selected, `list` will show all collections.
```

---

### Commands (within a selected collection)

```txt
list
    List all documents in the current collection.

count
    Show how many documents are in the current collection.

get <key.path> [where <sub_key> <is|not is|gr|ls> <value>]
    Retrieve the value of a key. Supports nested keys via dot notation.

set <key.path> <value>
    Set the value of a key. Type is automatically inferred.

del <key.path>
    Delete a key from the current document.

name
    Show the name of the selected collection.
```

---

## 🧪 Command Examples

```txt
new my_tasks
select my_tasks

set user.name "Juan"
set user.age 30
get user.name

get users.0.name where active is true
del user.age

commit
exit
```

---

## ⚙️ Running with Arguments

When passing command-line arguments, InfuseDB will **execute commands directly** and exit (non-interactive mode):

```sh
$ infusedb -p mydata.mdb -c tasks set task.name "Buy milk"
```

Common flags:

| Flag        | Description                                       |
|-------------|---------------------------------------------------|
| `-p <path>` | Path to the `.mdb` file. Default: `default.mdb`  |
| `-c <name>` | Name of the collection. Default: `default`       |
| `-s`        | (if built with `--features server`) start TCP server |

---

## 🔌 Server Mode (optional)

If compiled with the `server` feature, InfuseDB can run as a TCP server:

```sh
cargo run --features server -- -s
```

This starts a listener on `0.0.0.0:1234`. It's a starting point for remote command execution or a basic API.

---

## 📦 Internal Structure

- **infusedb/**: core database logic and types (`DataType`, `InfuseDB`, etc.)
- **command/**: per-collection commands (`get`, `set`, etc.)
- **arg_parser/**: minimalist CLI argument parser.
- **server/** *(optional)*: embedded TCP server.
- **help_const.rs**: CLI help text definitions.

---
