use pnet::datalink;
use uuid::Uuid;

fn get_mac_address() -> Option<[u8; 6]> {
    // Iterate through network interfaces
    for iface in datalink::interfaces() {
        // Skip loopback interfaces
        if !iface.is_up() || iface.is_loopback() {
            continue;
        }
        // Return the MAC address if available
        if let Some(mac) = iface.mac {
            return Some(mac.octets());
        }
    }
    None
}

pub fn generate_uuid_v1() -> Option<Uuid> {
    // Retrieve the MAC address of the first valid network interface
    let node_id = get_mac_address();
    match node_id {
        None => return None,
        Some(mac) => {
            // Generate a UUID v1 using the node ID
            return Some(Uuid::now_v1(&mac));
        }
    }
}
