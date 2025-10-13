<p align="center">
<img src="https://github.com/forcequitOS/netcaps/blob/main/netcaps.png?raw=true" width="30%">
</p>

<h2 align="center">A network activity light on your Caps Lock key.</h2>

**Try it today, you'll love it, or your money back!**

### Install
```
brew install forcequitOS/brew/netcaps
```

It couldn't get much simpler than this. 

---
### Run At Startup
**Globally:**

1. Run `sudo brew services start netcaps`
2. Grant Input Monitoring permissions in System Settings
3. Run `sudo brew services restart netcaps` (Or restart the computer)
---
**Only Current User:**
1. Run `brew services start netcaps`
2. Grant Input Monitoring permissions in System Settings
3. Run `brew services restart netcaps` (Or log out and log back in)

---
And you're off to the races!

>[!NOTE]
All functionality of your Caps Lock key is 100% preserved with netcaps. Also, netcaps is proudly written in Swift. Yay. 

>[!WARNING]
I don't know if this will impact your battery life or if it'll kill your Caps Lock key LED over time. Your mileage may vary. I'm not responsible if this somehow blows up your computer, but it probably shouldn't.

>[!TIP]
Want to monitor disk activity instead of network activity? Check out the sister program, discaps, [right here.](https://github.com/forcequitOS/discaps)

---
### Usage:

`netcaps [arguments]`

**Arguments:**

*Basic Functions*

--silent, -s	- Silences command-line output

--version, -v	- Displays the current version of netcaps

--help, -h		- Shows the help menu

*Include Specific Interfaces*

--local-included, -l	- listen on loX in addition to others

--utun-included, -u		- listen on utunX in addition to others

*ONLY Listen to Specific Interfaces*

--local-only, -L	- only listen on loX

--en-only, -E		- only listen on enX

--peers-only, -P 	- only listen on awdlX and llwX

--utun-only, -U		- only listen on utunX

That really. Is about it. Have fun. 