#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use blog_os::task::{Task, simple_executor::SimpleExecutor};
use blog_os::task::keyboard; // new

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

mod vga_buffer;
mod serial;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use blog_os::allocator; // new import
    use blog_os::memory::{self, BootInfoFrameAllocator};
    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // new
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses())); // new
    executor.run();
    
    // as before
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    blog_os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// #[no_mangle] // double _start defination
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    blog_os::init();

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

    // Note: The actual address might be different for you. Use the address that
    // your page fault handler reports.

    // new
    // let ptr = 0xdeadbeaf as *mut u32;
    // unsafe { *ptr = 42; }

    let ptr = 0x207623 as *mut u32;

    // read from a code page
    unsafe { let _x = *ptr; }
    println!("read worked");

    // write to a code page
    unsafe { *ptr = 42; }
    println!("write worked");

    // #[allow(unconditional_recursion)]
    // fn stack_overflow() {
    //     stack_overflow(); // for each recursion, the return address is pushed
    // }

    // // trigger a stack overflow
    // stack_overflow();
    // as before
    #[cfg(test)]
    test_main();

    println!("It did not crash!"); // Timer Interrupts, no handler, then double fault

    // loop {
    //     use blog_os::print;
    //     print!("-"); // deadlock at WRITER (Timer Interrupts use print too)
    // }

    blog_os::hlt_loop();
}

#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    // loop {
        
    // }

    blog_os::hlt_loop();
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        // test();
        test.run(); // new
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    // serial_print!("trivial assertion... ");
    assert_eq!(1, 1);
    // serial_println!("[ok]");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}