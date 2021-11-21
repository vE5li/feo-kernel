#![feature(allocator_api)]
#![feature(panic_info_message)]
#![feature(lang_items)]
#![feature(asm)]
#![no_builtins]
#![no_main]
#![no_std]

extern crate alloc;

#[macro_use]
mod peripherals;
mod memory;
mod graphics;

use core::alloc::Layout;
use core::panic::PanicInfo;
use peripherals::uart::*;
use peripherals::gpio::*;
use peripherals::mailbox::*;
//use alloc::string::String;

#[global_allocator]
static ALLOCATOR: memory::heap::Allocator = memory::heap::Allocator::new(0x60000, 0x20000);

#[no_mangle]
pub extern fn kernel_main() -> ! {

    peripherals::initialize();

    let mut framebuffer = graphics::initialize();

    log_line!("turning on status led");

    let mut message = Message::<20>::new();
    message.clear_tags();
    message.push_tag(MailboxTag::SetLEDStatus, &[42, 1]); // 42 = status
    message.push_tag(MailboxTag::SetLEDStatus, &[130, 1]); // 130 = power (state is inverted)
    message.push_end_tag();
    message.send(Channel::Tags);
    message.receive(Channel::Tags);

    success!("kernel initialized");

    // gpio test

    log_line!("starting gpio test");

    set_function(Pin::V5, Function::Output);
    set_function(Pin::V6, Function::Output);

    set_state(Pin::V5, true);
    set_state(Pin::V6, false);

    // graphics test

    log_line!("starting graphics test");

    framebuffer.draw_rectangle(500, 0, 30, 30, 0xAAAAAA, 0xAAAAAA);
    framebuffer.draw_text(500, 40, "i am rectangular", 0xAAAAAA);

    // heap allocation test

    log_line!("starting allocation test");

    let boxed = alloc::boxed::Box::new(50);

    if *boxed != 50 {
        panic!("incorrect value in box");
    }

    //let heap_string = String::from("i live on the heap");
    //framebuffer.draw_text(500, 40, &heap_string, 0xAAAAAA);

    success!("allocation test passed");

    // echo test

    log_line!("starting echo test");

    loop {
        let character = read_character_blocking();
        write_character_blocking(character);
    }
}

#[no_mangle]
#[lang = "oom"]
#[allow(improper_ctypes_definitions)]
pub extern fn oom(_layout: Layout) -> ! {
    error!("out of memory");
    loop {}
}

#[lang = "eh_personality"]
pub extern fn eh_personality() {
    error!("unwind");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log_line!("[ kernel ] [ panic ] fatal error");

    if let Some(location) = info.location() {
        // location.caller
        log_line!("file {}; line 0x{:x}; column 0x{:x}", location.file(), location.line(), location.column());
    }

    if let Some(message) = info.message() {
        peripherals::logger::log(*message);
    }
    loop {}
}