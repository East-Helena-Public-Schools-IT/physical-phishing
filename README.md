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

(All the names can be whatever you want)

1. Get a USB-drive

2. Create a folder `hidden`

3. Create a location ID: `phisher --generate "location name"`

4. Put at `.bat` file inside `/hidden` with the contents of `start "" http://localhost:3000/<id>`. With the id being the uuid from the previous command. (Obviously changing the ip from localhost when using in the wild)

5. Create a shortcut of `/hidden/request.bat` and put it on the USB's root (`/`)

6. In file explorer; set `/hidden` to be a hidden folder

7. Set the shortcut's icon to looks like another application (Done in `Properties` > `Change icon...`)

Voila! You can now "deploy" usb-drives and see where they end up! (Assuming that people try and open the "folder")

Or there is just the script `gen.ps1` which can do all the work for you.