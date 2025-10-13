### Important release with tons of fixes, efficiency improvements, and quality-of-life features
I try. 

Overall should be dramatically more stable and reliable I hope. Sleep/wake events should be recognized and accounted for, there's some semblance of error handling that should trigger IOHIDManager to be restarted, device disconnections and reconnections should be handled pretty gracefully, and CPU usage should be reduced.

- CPU usage should be significantly lower now. It's a difficult compromise between balancing maximum CPU usage and shortest blink duration. However I have gotten CPU usage down to a maximum of around 10% (At peak network activity) without HORRIBLY compromising minimum blink duration. Still, though, the shortest blink will be longer now than the shortest blink was previously. I hope all of the three netcaps users will understand. 
- When entering and resuming from sleep (At least when closing and reopening the lid of a MacBook), the IOHIDManager instance would go bad and pretty much stop functioning. I've tried to detect only the specific error code that indicates this (As IOHIDManager reports a lot of error codes, despite things working fine!)*, so a new IOHIDManager instance should be spawned now when this is detected.
- When a keyboard device is inaccessible (Typically due to being asleep), the program will rest for a little bit, this is similar to the other enhancement I made in the aim of better CPU usage in 1.5.0. 
- When entering sleep manually, not when closing the MacBook lid and such, now the LED should 100%, guaranteed, always, turn off automatically. I was relying on the system to pick up my slack, and while it DOES, it's not instantaneous, while this fix is.
- The network interface list is cached now, reloading every 30 seconds/if it's completely empty (never) to try and salvage some form of CPU usage.
- Priority has been reduced even further when running in silent mode. Will this help with the big CPU usage crisis? Probably not. 
- There's now a bunch of (6) command-line arguments to filter which arguments are listened on. Legitimately have at it. Command-line flags are also handled a decent bit better, but I still probably should have just used ArgumentParser or something. I think. I don't know. 

* IOHIDManager only really reports errors if you have Karabiner's virtual HID device running. Otherwise, it works fine. 

Potential future feature idea that I want to simply write down but have no promises on ever delivering: Command-line argument to set the time interval. This is pretty important, since for Bluetooth keyboards, the LED won't really turn on for short pulses at a certain value. 

<sub>I don't like Swift 6! @MainActor this, @MainActor that. Maybe I should actually rewrite this mess for Swift 6 as intended than just annotating everything with @MainActor... huh.</sub>