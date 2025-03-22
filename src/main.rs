use core_graphics::{
    display::{CGDisplay, CGPoint, CGRect},
    event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton},
    event_source::{CGEventSource, CGEventSourceStateID},
};
use std::{env, process::exit};

const KNOWN_ARGS: [&str; 4] = ["--back", "--no-click", "-h", "--help"];

// Look at https://github.com/koekeishiya/skhd for being able to bind this to a key
// https://github.com/matschik/mouse-macos/blob/main/src/main.cpp#L90-L98 for sending clicks?
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let unknown: Vec<_> = args
        .iter()
        .filter(|a| !KNOWN_ARGS.contains(&a.as_str()))
        .collect();
    if !unknown.is_empty() {
        eprintln!("unknown args: {unknown:?}");
        usage();
        exit(1);
    } else if args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
        exit(0);
    }

    let forward = !args.iter().any(|a| a == "--back");
    let click = !args.iter().any(|a| a == "--no-click");

    warp(forward, click);
}

fn usage() {
    println!("usage: mouse-mover [--back] [--no-click] [-h|--help]");
    println!("  move the mouse to an adjacent screen and click to set focus");
    println!("  pass --back to move to the previous screen instead of the next");
    println!("  pass --no-click to prevent left clicking to set focus");
}

fn warp(forward: bool, click: bool) -> Option<()> {
    let displays: Vec<_> = CGDisplay::active_displays()
        .ok()?
        .into_iter()
        .map(CGDisplay::new)
        .collect();

    if displays.len() == 1 {
        return Some(()); // no other displays to warp to
    }

    let current_pos = get_mouse_position()?;
    let i = displays
        .iter()
        .position(|&d| d.bounds().contains(&current_pos))?;

    let j = if forward {
        if i == displays.len() - 1 { 0 } else { i + 1 }
    } else if i == 0 {
        displays.len() - 1
    } else {
        i - 1
    };

    let p = mid_point(displays[j].bounds());
    CGDisplay::warp_mouse_cursor_position(p).ok()?;

    if click {
        left_click_at(p)?;
    }

    Some(())
}

fn mid_point(r: CGRect) -> CGPoint {
    let dw = r.size.width / 2.0;
    let dh = r.size.height / 2.0;
    let mut p = r.origin;
    p.x += dw;
    p.y += dh;

    p
}

// The CGEvent API is doc hidden so you'll need to read the source
//   https://github.com/servo/core-foundation-rs/blob/main/core-graphics/src/event.rs#L645

fn get_mouse_position() -> Option<CGPoint> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
    CGEvent::new(source).ok().map(|evt| evt.location())
}

fn left_click_at(p: CGPoint) -> Option<()> {
    for ty in [CGEventType::LeftMouseDown, CGEventType::LeftMouseUp] {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
        let evt = CGEvent::new_mouse_event(source, ty, p, CGMouseButton::Left).ok()?;
        evt.post(CGEventTapLocation::HID);
    }

    Some(())
}
