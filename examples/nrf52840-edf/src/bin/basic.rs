//! Basic side-by-side example of the Earliest Deadline First scheduler
//!
//! This test spawns a number of background "ambient system load" workers
//! that are constantly working, and runs two sets of trials.
//!
//! The first trial runs with no deadline set, so our trial task is at the
//! same prioritization level as the background worker tasks.
//!
//! The second trial sets a deadline, meaning that it will be given higher
//! scheduling priority than background tasks, that have no deadline set

#![no_std]
#![no_main]

use core::sync::atomic::{Ordering, compiler_fence};

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    embassy_nrf::init(Default::default());

    // Enable flash cache to remove some flash latency jitter
    compiler_fence(Ordering::SeqCst);
    embassy_nrf::pac::NVMC.icachecnf().write(|w| {
        w.set_cacheen(true);
    });
    compiler_fence(Ordering::SeqCst);

    //
    // Baseline system load tunables
    //

    // how many load tasks? More load tasks means more tasks contending
    // for the runqueue
    let tasks = 32;
    // how long should each task work for? The longer the working time,
    // the longer the max jitter possible, even when a task is prioritized,
    // as EDF is still cooperative and not pre-emptive
    //
    // 33 ticks ~= 1ms
    let work_time_ticks = 33;
    // what fraction, 1/denominator, should the system be busy?
    // bigger number means **less** busy
    //
    // 2  => 50%
    // 4  => 25%
    // 10 => 10%
    let denominator = 2;

    // Total time window, so each worker is working 1/denominator
    // amount of the total time
    let time_window = work_time_ticks * u64::from(tasks) * denominator;

    // Spawn all of our load workers!
    for i in 0..tasks {
        spawner.spawn(unwrap!(load_task(i, work_time_ticks, time_window)));
    }

    // Let all the tasks spin up
    defmt::println!("Spinning up load tasks...");
    Timer::after_secs(1).await;

    //
    // Trial task worker tunables
    //

    // How many steps should the workers under test run?
    // More steps means more chances to have to wait for other tasks
    // in line ahead of us.
    let num_steps = 100;

    // How many ticks should the worker take working on each step?
    //
    // 33 ticks ~= 1ms
    let work_ticks = 33;
    // How many ticks should the worker wait on each step?
    //
    // 66 ticks ~= 2ms
    let idle_ticks = 66;

    // How many times to repeat each trial?
    let trials = 3;

    // The total time a trial would take, in a perfect unloaded system
    let theoretical = (num_steps * work_ticks) + (num_steps * idle_ticks);

    defmt::println!("");
    defmt::println!("Starting UNPRIORITIZED worker trials");
    for _ in 0..trials {
        //
        // UNPRIORITIZED worker
        //
        defmt::println!("");
        defmt::println!("Starting unprioritized worker");
        let start = Instant::now();
        for _ in 0..num_steps {
            let now = Instant::now();
            while now.elapsed().as_ticks() < work_ticks {}
            Timer::after_ticks(idle_ticks).await;
        }
        let elapsed = start.elapsed().as_ticks();
        defmt::println!(
            "Trial complete, theoretical ticks: {=u64}, actual ticks: {=u64}",
            theoretical,
            elapsed
        );
        let ratio = ((elapsed as f32) / (theoretical as f32)) * 100.0;
        defmt::println!("Took {=f32}% of ideal time", ratio);
        Timer::after_millis(500).await;
    }

    Timer::after_secs(1).await;

    defmt::println!("");
    defmt::println!("Starting PRIORITIZED worker trials");
    for _ in 0..trials {
        //
        // PRIORITIZED worker
        //
        defmt::println!("");
        defmt::println!("Starting prioritized worker");
        let start = Instant::now();
        // Set the deadline to ~2x the theoretical time. In practice, setting any deadline
        // here elevates the current task above all other worker tasks.
        let meta = embassy_executor::Metadata::for_current_task().await;
        meta.set_deadline_after(theoretical * 2);

        // Perform the trial
        for _ in 0..num_steps {
            let now = Instant::now();
            while now.elapsed().as_ticks() < work_ticks {}
            Timer::after_ticks(idle_ticks).await;
        }

        let elapsed = start.elapsed().as_ticks();
        defmt::println!(
            "Trial complete, theoretical ticks: {=u64}, actual ticks: {=u64}",
            theoretical,
            elapsed
        );
        let ratio = ((elapsed as f32) / (theoretical as f32)) * 100.0;
        defmt::println!("Took {=f32}% of ideal time", ratio);

        // Unset the deadline, deadlines are not automatically cleared, and if our
        // deadline is in the past, then we get very high priority!
        meta.unset_deadline();

        Timer::after_millis(500).await;
    }

    defmt::println!("");
    defmt::println!("Trials Complete.");
}

#[embassy_executor::task(pool_size = 32)]
async fn load_task(id: u32, ticks_on: u64, ttl_ticks: u64) {
    let mut last_print = Instant::now();
    let mut last_tick = last_print;
    let mut variance = 0;
    let mut max_variance = 0;
    loop {
        let tgt = last_tick + Duration::from_ticks(ttl_ticks);
        assert!(tgt > Instant::now(), "fell too behind!");

        Timer::at(tgt).await;
        let now = Instant::now();
        // How late are we from the target?
        let var = now.duration_since(tgt).as_ticks();
        max_variance = max_variance.max(var);
        variance += var;

        // blocking work
        while now.elapsed().as_ticks() < ticks_on {}

        if last_print.elapsed() >= Duration::from_secs(1) {
            defmt::trace!(
                "Task {=u32} variance ticks (1s): {=u64}, max: {=u64}, act: {=u64}",
                id,
                variance,
                max_variance,
                ticks_on,
            );
            max_variance = 0;
            variance = 0;
            last_print = Instant::now();
        }

        last_tick = tgt;
    }
}
