// netcaps.swift
// Made by Taj C (forcequit)

import Foundation
import Darwin
import IOKit
import CoreGraphics
import AppKit

// MARK: - Caps Lock State
func isCapsLockOn() -> Bool {
    return CGEventSource.keyState(.combinedSessionState, key: 57)
}

@MainActor var legitInterval: TimeInterval = 0.00350
@MainActor var lastCapsState: Bool = false
@MainActor func setInterval() {
    let currentCapsState = isCapsLockOn()
    if currentCapsState != lastCapsState {
        legitInterval = currentCapsState ? 0.01050 : 0.00350
        lastCapsState = currentCapsState
    }
}

// MARK: - Blink Caps Lock LED
@MainActor
class CapsLockLEDManager {
    private var manager: IOHIDManager?
    private var cachedLEDElements: [(device: IOHIDDevice, element: IOHIDElement)] = []
    init?() {
        guard createManager() else { return nil }
    }
    private func createManager() -> Bool {
        manager = IOHIDManagerCreate(kCFAllocatorDefault, IOOptionBits(kIOHIDOptionsTypeNone))
        guard let manager = manager else { return false }
        let match: [[String: Any]] = [[
            kIOHIDDeviceUsagePageKey as String: kHIDPage_GenericDesktop,
            kIOHIDDeviceUsageKey as String: kHIDUsage_GD_Keyboard
        ]]
        IOHIDManagerSetDeviceMatchingMultiple(manager, match as CFArray)
        IOHIDManagerRegisterDeviceMatchingCallback(manager, deviceAttachedCallback, Unmanaged.passUnretained(self).toOpaque())
        IOHIDManagerRegisterDeviceRemovalCallback(manager, deviceRemovedCallback, Unmanaged.passUnretained(self).toOpaque())
        IOHIDManagerScheduleWithRunLoop(manager, CFRunLoopGetCurrent(), CFRunLoopMode.defaultMode.rawValue)
        let openRc = IOHIDManagerOpen(manager, IOOptionBits(kIOHIDOptionsTypeNone))
        if openRc == kIOReturnNotPermitted {
            print("Input Monitoring permissions need to be granted, exiting...")
            NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")!)
            exit(1)
        }
        return cacheDevices()
    }
    
    private func cacheDevices() -> Bool {
        guard let manager = manager,
              let devicesCF = IOHIDManagerCopyDevices(manager) else { return false }
        
        cachedLEDElements.removeAll()
        let devices = (devicesCF as NSSet) as! Set<IOHIDDevice>
        
        for device in devices {
            var ledPage = kHIDPage_LEDs
            let matchDict = NSMutableDictionary()
            matchDict[kIOHIDElementUsagePageKey] = CFNumberCreate(kCFAllocatorDefault, .intType, &ledPage)
            guard let elementsCF = IOHIDDeviceCopyMatchingElements(device, matchDict, 0) else {
                continue
            }
            let elements = elementsCF as! [IOHIDElement]
            for element in elements where IOHIDElementGetUsagePage(element) == UInt32(kHIDPage_LEDs) {
                if IOHIDElementGetUsage(element) == UInt32(kHIDUsage_LED_CapsLock) {
                    cachedLEDElements.append((device, element))
                }
            }
        }
        return !cachedLEDElements.isEmpty
    }
    
    func reinitialize() {
        if let manager = manager {
            IOHIDManagerUnscheduleFromRunLoop(manager, CFRunLoopGetCurrent(), CFRunLoopMode.defaultMode.rawValue)
            IOHIDManagerClose(manager, IOOptionBits(kIOHIDOptionsTypeNone))
        }
        manager = nil
        let _ = createManager()
    }
    
    func toggle(_ on: Bool) {
        for (device, element) in cachedLEDElements {
            let value = IOHIDValueCreateWithIntegerValue(
                kCFAllocatorDefault,
                element,
                mach_absolute_time(),
                on ? 1 : 0
            )
            let result = IOHIDDeviceSetValue(device, element, value)
            if result == 268435459 || result == -536870195 {
                if !silent {
                    print("Re-initializing IOHIDManager...")
                }
                reinitialize()
            }
        }
    }
    
    func blink(times: Int = 1, interval: TimeInterval) {
        let capsOn = isCapsLockOn()
        for _ in 1...times {
            if capsOn {
                toggle(false)
                Thread.sleep(forTimeInterval: interval)
                toggle(true)
            } else {
                toggle(true)
                Thread.sleep(forTimeInterval: interval)
                toggle(false)
            }
            Thread.sleep(forTimeInterval: interval)
        }
    }
    
    // This stuff should just recache everything upon attaching/detaching a device
    private let deviceAttachedCallback: IOHIDDeviceCallback = { context, result, sender, device in
        guard let context = context else { return }
        let this = Unmanaged<CapsLockLEDManager>.fromOpaque(context).takeUnretainedValue()
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 500_000_000)
            this.reinitialize()
        }
    }

    private let deviceRemovedCallback: IOHIDDeviceCallback = { context, result, sender, device in
        guard let context = context else { return }
        let this = Unmanaged<CapsLockLEDManager>.fromOpaque(context).takeUnretainedValue()
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 500_000_000)
            this.reinitialize()
        }
    }
}
@MainActor let ledManager = CapsLockLEDManager()
@MainActor
func blinkCapsLock(times: Int = 1, interval: TimeInterval = legitInterval) {
    ledManager?.blink(times: times, interval: interval)
}

// MARK: - Network Monitoring
@MainActor var allowedPrefixes: Set<String> = ["en", "pdp_ip", "awdl", "ap", "llw"]
@MainActor var cachedInterfaceNames: [String] = []
@MainActor var lastInterfaceCacheTime: TimeInterval = 0

@MainActor func refreshInterfaceCache() {
    var ifaddr: UnsafeMutablePointer<ifaddrs>?
    guard getifaddrs(&ifaddr) == 0, let firstAddr = ifaddr else { return }
    defer { freeifaddrs(ifaddr) }
    
    cachedInterfaceNames.removeAll(keepingCapacity: true)
    var ptr = firstAddr
    while true {
        let name = String(cString: ptr.pointee.ifa_name)
        if isAllowedInterface(name) && !cachedInterfaceNames.contains(name) {
            cachedInterfaceNames.append(name)
        }
        guard let next = ptr.pointee.ifa_next else { break }
        ptr = next
    }
}

@MainActor func isAllowedInterface(_ name: String) -> Bool {
    if name.hasPrefix("en") { return allowedPrefixes.contains("en") }
    if name.hasPrefix("lo") { return allowedPrefixes.contains("lo") }
    if name.hasPrefix("utun") { return allowedPrefixes.contains("utun") }
    if name.hasPrefix("awdl") { return allowedPrefixes.contains("awdl") }
    if name.hasPrefix("llw") { return allowedPrefixes.contains("llw") }
    if name.hasPrefix("ap") { return allowedPrefixes.contains("ap") }
    if name.hasPrefix("pdp_ip") { return allowedPrefixes.contains("pdp_ip") }
    return false
}

@MainActor func getNetworkBytes() -> (rx: UInt64, tx: UInt64) {
    let currentTime = ProcessInfo.processInfo.systemUptime
    if currentTime - lastInterfaceCacheTime > 30.0 || cachedInterfaceNames.isEmpty {
        refreshInterfaceCache()
        lastInterfaceCacheTime = currentTime
    }
    var ifaddr: UnsafeMutablePointer<ifaddrs>?
    guard getifaddrs(&ifaddr) == 0, let firstAddr = ifaddr else { return (0, 0) }
    defer { freeifaddrs(ifaddr) }
    var rx: UInt64 = 0
    var tx: UInt64 = 0
    var ptr = firstAddr
    while true {
        let name = String(cString: ptr.pointee.ifa_name)
        if isAllowedInterface(name) {
            if let data = ptr.pointee.ifa_data?.assumingMemoryBound(to: if_data.self) {
                rx &+= UInt64(data.pointee.ifi_ibytes)
                tx &+= UInt64(data.pointee.ifi_obytes)
            }
        }
        
        if ptr.pointee.ifa_next == nil { break }
        ptr = ptr.pointee.ifa_next!
    }
    return (rx, tx)
}

// This is really goofy.
let args = CommandLine.arguments
let silent = args.contains("-s") || args.contains("--silent")

@main
struct main {
    static func main() {
        // MARK: - Argument Handling
        // I probably should've just used ArgumentParser
        let hasHelpOrVersion = args.contains("-h") || args.contains("--help") ||
                               args.contains("-v") || args.contains("--version")
        if hasHelpOrVersion && args.count > 2 {
            print("")
            print("Error: --help and --version cannot be combined with other flags.")
            print("")
            exit(1)
        }

        if args.contains("-v") || args.contains("--version") {
            print("netcaps version 1.6.0")
            print("    Made by Taj C (forcequit)")
            print("    Check this out on GitHub, at https://github.com/forcequitOS/netcaps")
            exit(0)
        }

        let onlyFlags = [
            args.contains("-L") || args.contains("--local-only"),
            args.contains("-E") || args.contains("--en-only"),
            args.contains("-P") || args.contains("--peers-only"),
            args.contains("-U") || args.contains("--utun-only")
        ]
        let onlyFlagCount = onlyFlags.filter { $0 }.count

        let hasIncludedFlags = args.contains("-l") || args.contains("--local-included") ||
                               args.contains("-u") || args.contains("--utun-included")
        let hasOnlyFlags = onlyFlagCount > 0

        if (onlyFlagCount > 1) || (hasIncludedFlags && hasOnlyFlags) || args.contains("-h") || args.contains("--help") {
            print("")
            print("Usage:")
            print("    netcaps [arguments]")
            print("")
            print("Arguments:")
            print("    --silent, -s         - silences command-line output")
            print("    --local-only, -L     - only listen on loX")
            print("    --en-only, -E        - only listen on enX")
            print("    --peers-only, -P     - only listen on awdlX and llwX")
            print("    --utun-only, -U      - only listen on utunX")
            print("    --local-included, -l - listen on loX in addition to others")
            print("    --utun-included, -u  - listen on utunX in addition to others")
            print("    --version, -v        - displays current version of netcaps")
            print("    --help, -h           - shows this help menu")
            print("")
            if onlyFlagCount > 1 {
                print("Error: Cannot use multiple 'only' flags together.")
                print("")
            } else if hasIncludedFlags && hasOnlyFlags {
                print("Error: Cannot combine 'included' flags with 'only' flags.")
                print("")
            }
            exit(0)
        }

        if args.contains("-L") || args.contains("--local-only") {
            allowedPrefixes = ["lo"]
        } else if args.contains("-E") || args.contains("--en-only") {
            allowedPrefixes = ["en"]
        } else if args.contains("-P") || args.contains("--peers-only") {
            allowedPrefixes = ["awdl", "llw"]
        } else if args.contains("-U") || args.contains("--utun-only") {
            allowedPrefixes = ["utun"]
        } else if args.contains("-l") || args.contains("--local-included") {
            allowedPrefixes.insert("lo")
        } else if args.contains("-u") || args.contains("--utun-included") {
            allowedPrefixes.insert("utun")
        }
        
        if silent {
            setpriority(PRIO_PROCESS, 0, 15)
        }
        var previousBytes = getNetworkBytes()
        var checksWithoutActivity = 0
        let maxChecksBeforeSlowdown = 7500
        
        // MARK: - Main Loop
        while true {
            autoreleasepool {
                setInterval()
                Thread.sleep(forTimeInterval: legitInterval)
                let currentBytes = getNetworkBytes()
                if currentBytes.rx > previousBytes.rx || currentBytes.tx > previousBytes.tx {
                    if !silent {
                        print("RX: \(currentBytes.rx), TX: \(currentBytes.tx)")
                    }
                    blinkCapsLock()
                    checksWithoutActivity = 0
                } else {
                    checksWithoutActivity += 1
                    if checksWithoutActivity >= maxChecksBeforeSlowdown {
                        Thread.sleep(forTimeInterval: 0.05)
                    }
                }
                previousBytes = currentBytes
            }
        }
    }
}

class SleepObserver {
    init() {
        let center = NSWorkspace.shared.notificationCenter
        center.addObserver(
            self,
            selector: #selector(willSleep(_:)),
            name: NSWorkspace.willSleepNotification,
            object: nil
        )
        center.addObserver(
            self,
            selector: #selector(didWake(_:)),
            name: NSWorkspace.didWakeNotification,
            object: nil
        )
    }
    @MainActor @objc func willSleep(_ notification: Notification) {
        ledManager?.toggle(false)
    }
    @MainActor @objc func didWake(_ notification: Notification) {
        Task {
            try? await Task.sleep(nanoseconds: 500_000_000)
            ledManager?.reinitialize()
        }
    }
}
