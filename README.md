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
name = '<printer name>'

# Comment out or remove if trello is not needed
[trello]
app_key = '<APP KEY>'
token = '<USER TOKEN>'
# Tickets are filtered using the following label
# Label is removed after printing
print_label = '<LABEL>'
# Use an empty array to search all boards
limit_to_boards = ["Example Board"]

# Comment out or remove if jira is not needed
[jira]
# Hostname only. Http is not supported
# Example: test.atlassian.com
host = '<JIRA HOSTNAME>'
user = '<USERNAME OR MAIL>'
token = '<USER TOKEN>'
# Tickets are filtered using the following label
# Label is removed after printing
print_label = '<LABEL>'
# Use an empty array to search for all types
limit_to_types = ["Issue"]
# Use an empty array to search all boards
limit_to_projects = ["Example Board"]
```

There is also an example configuration available in the
config folder. It is not necessary to define both jira and
trello. Simply remove the configuration part for the service
you don't want to use.

There are three different locations for the configuration file.
The system always takes a look at all four locations merging
them together in the following order. Higher number overrides
the conifguration entry of a lower number.
  1. `/etc/ticket_printer/ticket_printer.{ext}`
  2. `~/.config/ticket_printer/ticket_printer.{ext}`
  3. `./ticket_printer.{ext}`
  4. Environment Variables
  5. Command Line Parameters

Possible extensions are json, toml and yaml.

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
