#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    entry_point, BootInfo,
};
use log::info;
use core::panic::PanicInfo;
use kernel::memory::active_level_4_table;
use kernel::task::Task;
use kernel::{println, serial_println};
use x86_64::structures::paging::PageTable;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use kernel::allocator;
    use kernel::memory;
    use kernel::task::executor::Executor;
    use kernel::task::keyboard;
    use memory::BootInfoFrameAllocator;
    use x86_64::VirtAddr;

    let physical_memory_offset = boot_info
        .physical_memory_offset
        .take()
        .expect("physical_memory_offset not set");

    info!("physical_memory_offset: {:#x}", physical_memory_offset);
    
    unsafe {
        serial_println!(
            "{:X}",
            active_level_4_table(VirtAddr::new(physical_memory_offset)) as *const PageTable
                as usize
        );
        serial_println!(
            "{:X}",
            active_level_4_table(VirtAddr::new(physical_memory_offset)) as *const PageTable
                as usize
        );
        let a = 0;
        serial_println!("{:X}", &a as *const i32 as usize);
    }
    kernel::init();

    let mut mapper = unsafe { memory::init(VirtAddr::new(physical_memory_offset)) };
    println!("{:?}", boot_info.memory_regions);
    println!("\n");
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    println!("{:?}", boot_info.memory_regions);

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
    use kernel::hlt_loop;
    println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}
