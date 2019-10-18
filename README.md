# ticket_printer

A program for printing jira or trello tickets on small cards

## Configuration

There is a limited configuration available to supply
credentials for jira or trello and to allow a rudimentary
personalisation of the printed cards.

```toml
[pdf]
height = 62.0
width = 100.0
margin = 4.0
title_lines = 2
title_seperator_margin = 4.0
qrcode_seperator_margin = 4.0
subtitle_size = 4.0

[printer]
media = 'Custom.62x100m'
orientation = 'landscape'
number_of_copies = 2
name = 'Brother_QL_700'

[trello]
app_key = '<APP KEY>'
token = '<USER TOKEN>'
# Tickets are filtered using the following label
# Label is removed after printing
print_label = '<LABEL>'
# Use an empty array to search all boards
limit_to_boards = ["Example Board"]

[jira]
# Hostname only. Http is not supported
# Example: test.atlassian.com
host = '<JIRA HOSTNAME>'
user = '<USERNAME OR MAIL>'
token = '<USER TOKEN>'
# Tickets are filtered using the following label
# Label is removed after printing
print_label = '<LABEL>'
# Use an empty array to search all boards
limit_to_projects = ["Example Board"]
```

There is also an example configuration available in the
config folder. It is not necessary to define both jira and
trello. Simply remove the configuration part for the service
you don't want to use.

The location of the configuration file is system dependent
and can be found here:
<https://docs.rs/directories/0.10.0/directories/struct.ProjectDirs.html#method.config_dir>

## qrcode

The qrcode contains a directlink to the trello or jira
ticket. It will, however, only show up if the url is small
enough. In the default configuration, the url size may not
be bigger then 23 lettes.

## Installation

On a system with rust installed you can install this package
using:

```sh
cargo install https://github.com/mettke/ticket-printer
```

Otherwise you may want to try a binary distributed with each
version


License: MIT OR Apache-2.0
