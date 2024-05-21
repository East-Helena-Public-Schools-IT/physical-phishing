# Usage

To generate locations for you to use later:

```sh
phisher --generate "Some Location"
```

To run the server:

```sh
phisher --serve
```

Then, save the generated pairs to a file for later use. Put the ids (uuid) as a link `whatever.ip:port/<uuid>` then you can decide how to get users to (hopefully not) run a script which pings that url.

## How we are using

Set up a server that is accessible to your whole environment. Then run the `gen.ps1` script, following the prompts. "Deploy" the usb drives around your environment. 

## gen.ps1

The script does these steps:

1. Get the path to a USB drive
2. Create a hidden folder (system + hidden flags)
3. Create a powershell script inside the hidden folder
4. Create a shortcut to the hidden script
	4.1 Make the shortcut actually run powershell, and then point it at the hidden script
	4.2 Change the icon of the shortcut to look like a folder
5. Done!
