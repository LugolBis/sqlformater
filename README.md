# sqlformater

**sqlformater** is a command-line tool (CLI) built in Rust, designed to **automatically format SQL scripts**. It supports individual files or entire directories and allows customized configuration via settings files.

## 🚀 Getting Started

Install with Cargo :
```bash
cargo install --git https://github.com/LugolBis/sqlformater.git
```

Install with pip :
```bash
pip install sqlformater
```

Usage :
```bash
sqlformater <PATHS> [OPTIONS]
```

## 📂 <PATHS>: Specify the files or folders to format

You can provide **one or more** of the following:

- One or more **folder paths**: formats all `.sql` files in the subdirectories.
- One or more **`.sql` files**: formats specific individual files.
- `.` or `*`: selects the **current directory** and all its subdirectories.

## ⚙️ Available Options

|Option|Description|
|:-|:-|
|`-logs_path=<FOLDER_PATH>`,<br>`--logs_path=<FOLDER_PATH>`|Specifies the folder where **logs** will be saved.|
|`-settings_path=<PATH>`,<br>`--settings_path=<PATH>`|Path to the **configuration file** or the folder that contains it.|
| `-verbose`,<br>`--verbose`| Enables **verbose mode** to display detailed execution information. |
| `-status`,<br>`--status`| Shows **diagnostic information**: settings, logs, etc. |
| `-help`,<br>`--help` | Displays **general help**. |
| `-help-settings`,<br>`--help-settings`| Displays **information about available configuration settings**. |

## 📘 Examples

Format all SQL files in the current directory and subdirectories:
```bash
sqlformater .
```

Format two specific files:
```bash
sqlformater tutu/queries/init.sql tutu/queries/update_users.sql
```

Format all files in a folder with a specific config file:
```bash
sqlformater home/tutu/migrations --settings_path=./config/settings.json
```

Enable verbose mode:
```bash
sqlformater * --verbose
```

## 🔧 Configuration

You can customize the formatter's behavior via a configuration file. To see the available settings:

```bash
sqlformater --help-settings
```
