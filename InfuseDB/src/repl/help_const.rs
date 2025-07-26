pub const HELP_STR_COL: &'static str = r"Collection commands:

  list
      List all documents in the selected collection.

  count
      Show how many documents exist in the current collection.

  get <key.path> [where <sub_key> <is|not is|gr|ls> <value>]
      Retrieve the value associated with a key. Nested keys can be accessed with dot notation (e.g., `user.name` or `users.0.name`).
      You can also filter results using where, like: get todo_list where done is true.

  set <key.path> <value>
      Set the value of a key. Supports nested keys with dot notation. Value type is inferred automatically (string, number, bool, etc).

  del <key.path>
      Delete a key or nested key from the current collection.

  name
      Show the name of the currently selected collection.";

pub const HELP_STR_MAIN: &'static str = r"Available commands:

    help
        Show this help message.

    exit
        Exit the application.

    list
        List all available collections.

    select <collection_name>
        Select a collection to work with. You must provide a valid collection name.

    new <collection_name>
        Create a new collection with the given name.

    del_col <collection_name>
        Delete the selected collection.
        If no collection is selected, you must provide the name of the collection to delete.

    commit
        Save all changes made to the database.

    Notes:
    - You must select a collection using 'select' before performing actions on it.
    - If a collection is not selected, 'list' will show all available collections.";
