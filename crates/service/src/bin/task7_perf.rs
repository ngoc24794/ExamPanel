//! Task 7 Performance Benchmark: solve_hard and 8x200k optimize on Q-shaped demo.

use exam_panel_core::domain::fixtures::{make_canonical_q_problem, QVariant};
use exam_panel_core::optimize::{optimize, Budget, OptimizeOptions};
use exam_panel_core::solver::{solve_hard, SolveOptions};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let problem = make_canonical_q_problem(QVariant::SyntheticCampuses);

    println!("===============================================================================");
    println!("TASK 7 PERFORMANCE BENCHMARK REPORT");
    println!("===============================================================================\n");
    println!("Platform Information:");
    println!("  - Target: x86_64-pc-windows-msvc");
    println!(
        "  - Hardware Concurrency: {} logical cores",
        std::thread::available_parallelism().map_or(1, |n| n.get())
    );
    println!("  - Problem: Q-shaped dataset (4 synthetic campuses, 12 teachers, 2 subjects, 4 exams, 24 panels, 60 seats)\n");

    // -------------------------------------------------------------------------
    // 1. solve_hard Benchmark (5 runs)
    // -------------------------------------------------------------------------
    println!("## 1. Stage 1: solve_hard Performance");
    let mut solve_times = Vec::new();
    for i in 1..=5 {
        let hard_opts = SolveOptions {
            seed: 42 + i as u64,
            time_limit_ms: 3000,
            max_nodes: 1_000_000,
        };
        let start = Instant::now();
        let sol = solve_hard(&problem, &hard_opts)?;
        let elapsed = start.elapsed();
        solve_times.push(elapsed);
        println!(
            "  - Run {}: {:.3} ms ({} assignments, {} nodes)",
            i,
            elapsed.as_secs_f64() * 1000.0,
            sol.assignments.len(),
            sol.stats.nodes
        );
    }

    solve_times.sort();
    let solve_min = solve_times.first().unwrap().as_secs_f64() * 1000.0;
    let solve_med = solve_times[solve_times.len() / 2].as_secs_f64() * 1000.0;
    let solve_max = solve_times.last().unwrap().as_secs_f64() * 1000.0;
    println!("solve_hard Summary (5 runs):");
    println!("  - Min:    {:.3} ms", solve_min);
    println!("  - Median: {:.3} ms", solve_med);
    println!("  - Max:    {:.3} ms\n", solve_max);

    // -------------------------------------------------------------------------
    // 2. optimize Benchmark: R = 8 runs, 200,000 iterations each (3 runs)
    // -------------------------------------------------------------------------
    println!("## 2. Stage 2: optimize (R = 8, 200,000 iterations per run = 1,600,000 iters)");
    let opts = OptimizeOptions {
        base_seed: 42,
        num_runs: 8,
        budget: Budget::Iterations(200_000),
        max_plans: 3,
        diversity_threshold: 0.20,
        initial_assignments: None,
        cancel: None,
        progress: None,
    };

    let mut opt_times = Vec::new();
    let mut opt_scores = Vec::new();

    for i in 1..=3 {
        eprintln!("Running optimize benchmark run {}/3...", i);
        let start = Instant::now();
        let res = optimize(&problem, &opts)?;
        let elapsed = start.elapsed();
        opt_times.push(elapsed);
        let score = res.plans[0].report.total;
        opt_scores.push(score);
        let throughput = 1_600_000.0 / elapsed.as_secs_f64();
        println!(
            "  - Run {}: {:.3} s (Best score: {:.2}, Throughput: {:.0} iters/s)",
            i,
            elapsed.as_secs_f64(),
            score,
            throughput
        );
    }

    opt_times.sort();
    let opt_min = opt_times.first().unwrap().as_secs_f64();
    let opt_med = opt_times[opt_times.len() / 2].as_secs_f64();
    let opt_max = opt_times.last().unwrap().as_secs_f64();
    println!("\noptimize Summary (3 runs of 8 x 200k iterations):");
    println!("  - Min:    {:.3} s", opt_min);
    println!("  - Median: {:.3} s", opt_med);
    println!("  - Max:    {:.3} s", opt_max);
    println!("  - Target: 1.500 s");

    if opt_med <= 1.5 {
        println!(
            "  - Status: PASSED (median {:.3} s <= 1.5 s target)",
            opt_med
        );
    } else {
        println!(
            "  - Status: EXCEEDS 1.5s TARGET (measured median: {:.3} s; proposed target: {:.2} s)",
            opt_med,
            (opt_med * 1.2).ceil()
        );
    }

    Ok(())
}
