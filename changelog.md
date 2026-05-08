### Rust Rewrite

- Replaced the SwiftPM/Xcode executable with a Cargo-based Rust implementation.
- Kept the same Caps Lock LED, network-interface monitoring, and command-line behavior.

### Efficiency & Leniency

- The default interval's been changed again to be LESS efficient, because I've done some magic tricks to decrease CPU usage further, and CPU usage was already really low. 
- Caps Lock state is checked only every 500ms now
- A per-device cooldown been introduced when failing due to the infamous exclusive access error
- When a super specific error (that still indicates a success) which I've only seen caused when using Karabiner is detected, any other devices will be skipped
- Only the first LED will be enumerated for a given device
- CPU usage should be somewhat reduced now if you use Karabiner-Elements. Your CPU usage for netcaps WILL ALWAYS be better, though, if you disable Karabiner's virtual HID device. The peak CPU usage I've gotten WITH Karabiner on this release is ~10%. Without, it's around 6%. 

Future: Don't re-enumerate duplicate devices maybe or maybe not depending on if I can be bothered.
