#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_os::println;
use rust_os::task::Task;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use memory::BootInfoFrameAllocator;
    use rust_os::allocator;
    use rust_os::memory;
    use rust_os::task::executor::Executor;
    use rust_os::task::keyboard;
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    rust_os::init();

    let mut mapper = unsafe { memory::init(VirtAddr::new(boot_info.physical_memory_offset)) };

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));

    executor.spawn(Task::new(async_test_main()));

    executor.run();
}

async fn async_test_main() {
    #[cfg(test)]
    test_main();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use rust_os::hlt_loop;
    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}
