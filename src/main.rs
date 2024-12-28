// use std::collections::HashSet;
// use std::collections::BinaryHeap;
// use std::cmp::Reverse;

use std::error::Error;
use std::process::exit;

use x11rb::connection::Connection;
use x11rb::errors::{ReplyError, ConnectionError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::{ErrorKind, Event};


#[derive(Debug)]
struct WindowState {
    window: Window,
    frame_window: Window,
    x: i16,
    y: i16,
    width: u16,    
}

impl<'a, C: Connection> WmState<'a, C> {
    pub fn new(conn: &'a C, screen_num: usize) -> WmState<'a, C> {
        // let screen = &conn.setup().roots[screen_num];
        WmState {
            conn,
            screen_num,
            windows: Vec::default()
        }
    }

    pub fn handle_configure_request(&self, event: ConfigureRequestEvent) -> Result<(), ReplyError> {
        let aux = ConfigureWindowAux::from_configure_request(&event);
        println!("Configure: {:?}", aux);

        self.conn.configure_window(event.window, &aux)?;
        Ok(())
    }


    pub fn handle_map_request(&self, event: MapRequestEvent) -> Result<(), ReplyError> {
        // frame before mapping

        self.conn.map_window(event.window)?;

        Ok(())
    }
}


#[derive(Debug)]
struct WmState<'a, C: Connection> {
    conn: &'a C, // the lifetime of this reference must not outlive the original connection
    screen_num: usize,
    // black_gc: Gcontext,
    windows: Vec<WindowState>,
    // pending_expose: HashSet<Window>,
    // wm_protocols: Atom,
    // wm_delete_window: Atom,
    // sequences_to_ignore: BinaryHeap<Reverse<u16>>,
    // If this is Some, we are currently dragging the given
    // window with the given offset relative
    // to the mouse.
    // drag_window: Option<(Window, (i16, i16))>,
}


fn attach_window_manager<C: Connection>(conn: &C, screen: &Screen) -> Result<(), ReplyError> {
    // Try to become the current window manager. If another wm is running this will fail
    let changes = ChangeWindowAttributesAux::new()
        .event_mask(EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY);

    let res = conn.change_window_attributes(screen.root, &changes)?.check();

    match res {
        Err(ReplyError::X11Error(error)) => { 
            if error.error_kind == ErrorKind::Access {
                eprintln!("Another WM is already running.");
            }
            exit(1);
        }
        Err(ReplyError::ConnectionError(error)) => {
            eprintln!("Failed to connect to X11 server. {}", error);
            exit(1);
        }
        Ok(()) => {
            res
        }
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    attach_window_manager(&conn, &screen)?;

    let wm_state = WmState::new(&conn, screen_num);

    // Enter main event loop
    loop {
        let event = conn.wait_for_event()?;

        match event {
            Event::ConfigureRequest(event)  => { wm_state.handle_configure_request(event)?; }
            Event::MapRequest(event)        => { wm_state.handle_map_request(event)?; }
            // Event::UnmapNotify(event)       => {}
            // Event::ConfigureNotify(event)  => {}
            // Event::MapNotify(event)        => {}
            // Event::CreateNotify(event)      => {}
            // Event::DestroyNotify(event)     => {}
            // Event::ReparentNotify(event)    => {}
            _ => { }
        };

        // std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
