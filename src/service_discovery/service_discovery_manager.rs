use bindings::avahi;
use service_discovery::callback_handler::CallbackHandler;
use service_discovery::callback_handler::DiscoveryEventHandler;
use service_discovery::callback_handler::ClientReference;
use service_discovery::callback_handler::ServiceDescription;
use service_discovery::callback_handler::BrowsedServiceDescription;

use libc::{c_void, free};
use std::mem;

use std::ffi::CString;
use std::ffi::CStr;
use std::ptr;

pub struct ServiceDiscoveryManager;

impl ServiceDiscoveryManager {
    pub fn new() -> ServiceDiscoveryManager {
        ServiceDiscoveryManager
    }

    /// List all available service by type_name.
    pub fn discover_services(&self, service_type: &str) {
        unsafe {
            let mut client_error_code: i32 = 0;

            let simple_poll = avahi::avahi_simple_poll_new();
            let poll = avahi::avahi_simple_poll_get(simple_poll);

            let client =
                avahi::avahi_client_new(poll,
                                        avahi::AvahiClientFlags::AVAHI_CLIENT_IGNORE_USER_CONFIG,
                                        *Box::new(CallbackHandler::client_callback),
                                        ptr::null_mut(),
                                        &mut client_error_code);

            // Check that we've created client successfully, otherwise try to resolve error
            // into human-readable string.
            if client.is_null() {
                let error_string = CStr::from_ptr(avahi::avahi_strerror(client_error_code));
                free(client as *mut c_void);
                panic!("Failed to create avahi client: {}",
                       error_string.to_str().unwrap());

            }

            let client_reference = ClientReference {
                client: client,
                handler: self,
            };

            // Let's search for service of requested type.
            let sb = avahi::avahi_service_browser_new(client,
                                                      avahi::AvahiIfIndex::AVAHI_IF_UNSPEC,
                                                      avahi::AvahiProtocol::AVAHI_PROTO_UNSPEC,
                                                      CString::new(service_type).unwrap().as_ptr(),
                                                      ptr::null_mut(),
                                                      avahi::AvahiLookupFlags::AVAHI_LOOKUP_UNSPEC,
                                                      *Box::new(CallbackHandler::browse_callback::<ServiceDiscoveryManager>),
                                                      mem::transmute(&client_reference));

            avahi::avahi_simple_poll_loop(simple_poll);

            avahi::avahi_service_browser_free(sb);
            avahi::avahi_client_free(client);
            avahi::avahi_simple_poll_free(simple_poll);
        }
    }
}

impl DiscoveryEventHandler for ServiceDiscoveryManager {
    fn on_service_discovered(&self, service_description: BrowsedServiceDescription) {
        println!("Service browsed: {:?}", service_description);
    }

    fn on_service_resolved(&self, service_description: ServiceDescription) {
        println!("Service resolved: {:?}", service_description);
    }
}