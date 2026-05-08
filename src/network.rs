use std::collections::HashSet;
use std::ffi::CStr;
use std::time::{Duration, Instant};

const INTERFACE_CACHE_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InterfaceKind {
    Ap,
    Awdl,
    Ethernet,
    Llw,
    Loopback,
    PdpIp,
    Utun,
}

#[derive(Clone)]
pub struct InterfaceFilter {
    allowed: HashSet<InterfaceKind>,
}

impl Default for InterfaceFilter {
    fn default() -> Self {
        Self::only([
            InterfaceKind::Ethernet,
            InterfaceKind::PdpIp,
            InterfaceKind::Awdl,
            InterfaceKind::Ap,
            InterfaceKind::Llw,
        ])
    }
}

impl InterfaceFilter {
    pub fn only(kinds: impl IntoIterator<Item = InterfaceKind>) -> Self {
        Self {
            allowed: kinds.into_iter().collect(),
        }
    }

    pub fn insert(&mut self, kind: InterfaceKind) {
        self.allowed.insert(kind);
    }

    pub(crate) fn allows_name(&self, name: &str) -> bool {
        InterfaceKind::from_name(name).is_some_and(|kind| self.allowed.contains(&kind))
    }
}

impl InterfaceKind {
    fn from_name(name: &str) -> Option<Self> {
        if name.starts_with("pdp_ip") {
            return Some(Self::PdpIp);
        }
        if name.starts_with("utun") {
            return Some(Self::Utun);
        }
        if name.starts_with("awdl") {
            return Some(Self::Awdl);
        }
        if name.starts_with("llw") {
            return Some(Self::Llw);
        }
        if name.starts_with("lo") {
            return Some(Self::Loopback);
        }
        if name.starts_with("en") {
            return Some(Self::Ethernet);
        }
        if name.starts_with("ap") {
            return Some(Self::Ap);
        }
        None
    }
}

#[derive(Clone, Copy)]
pub struct ByteCounts {
    pub rx: u64,
    pub tx: u64,
}

impl ByteCounts {
    pub fn has_activity_since(self, previous: Self) -> bool {
        self.rx > previous.rx || self.tx > previous.tx
    }
}

pub struct NetworkMonitor {
    filter: InterfaceFilter,
    cached_interface_names: HashSet<String>,
    last_cache_refresh: Option<Instant>,
}

impl NetworkMonitor {
    pub fn new(filter: InterfaceFilter) -> Self {
        Self {
            filter,
            cached_interface_names: HashSet::new(),
            last_cache_refresh: None,
        }
    }

    pub fn byte_counts(&mut self) -> ByteCounts {
        if self.cache_expired() {
            self.refresh_interface_cache();
        }

        unsafe { read_network_bytes(&self.cached_interface_names) }
    }

    fn cache_expired(&self) -> bool {
        self.cached_interface_names.is_empty()
            || self
                .last_cache_refresh
                .is_none_or(|last_refresh| last_refresh.elapsed() > INTERFACE_CACHE_INTERVAL)
    }

    fn refresh_interface_cache(&mut self) {
        self.cached_interface_names = unsafe { read_interface_names(&self.filter) };
        self.last_cache_refresh = Some(Instant::now());
    }
}

unsafe fn read_interface_names(filter: &InterfaceFilter) -> HashSet<String> {
    let mut ifaddr = std::ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut ifaddr) } != 0 || ifaddr.is_null() {
        return HashSet::new();
    }

    let mut names = HashSet::new();
    let mut current = ifaddr;
    while !current.is_null() {
        if let Some(name) = unsafe { interface_name(current) }
            && filter.allows_name(&name)
        {
            names.insert(name);
        }
        current = unsafe { (*current).ifa_next };
    }

    unsafe {
        libc::freeifaddrs(ifaddr);
    }
    names
}

unsafe fn read_network_bytes(interface_names: &HashSet<String>) -> ByteCounts {
    let mut ifaddr = std::ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut ifaddr) } != 0 || ifaddr.is_null() {
        return ByteCounts { rx: 0, tx: 0 };
    }

    let mut rx = 0_u64;
    let mut tx = 0_u64;
    let mut current = ifaddr;

    while !current.is_null() {
        if let Some(name) = unsafe { interface_name(current) }
            && interface_names.contains(&name)
        {
            let data = unsafe { (*current).ifa_data as *const libc::if_data };
            if !data.is_null() {
                rx = rx.wrapping_add(unsafe { (*data).ifi_ibytes as u64 });
                tx = tx.wrapping_add(unsafe { (*data).ifi_obytes as u64 });
            }
        }
        current = unsafe { (*current).ifa_next };
    }

    unsafe {
        libc::freeifaddrs(ifaddr);
    }
    ByteCounts { rx, tx }
}

unsafe fn interface_name(ifaddr: *mut libc::ifaddrs) -> Option<String> {
    let name = unsafe { (*ifaddr).ifa_name };
    if name.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(name) }
        .to_str()
        .ok()
        .map(ToOwned::to_owned)
}
