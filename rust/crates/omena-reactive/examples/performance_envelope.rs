use std::{
    error::Error,
    time::{Duration, Instant},
};

use omena_reactive::{
    ChangePolicyV0, ReactiveEngineV0, ReactiveGraphBuilderV0, ReactiveNodeIdV0, ReactiveStateV0,
    ReactiveValueV0, StabilizeStatusV0,
};

const INPUT_COUNT: usize = 256;
const EVENT_COUNT: usize = 512;
const P95_CEILING: Duration = Duration::from_millis(250);
const MAX_CEILING: Duration = Duration::from_millis(750);

fn project_counter(state: &ReactiveStateV0) -> ReactiveStateV0 {
    match state {
        ReactiveStateV0::Available(ReactiveValueV0::Counter(value)) => {
            ReactiveStateV0::available(ReactiveValueV0::Counter(value.rotate_left(7)))
        }
        _ => ReactiveStateV0::unavailable("counterRequired", "expected a counter deposit"),
    }
}

fn settle(engine: &mut ReactiveEngineV0) -> Result<(), Box<dyn Error>> {
    loop {
        if matches!(
            engine.stabilize_step(4_096)?,
            StabilizeStatusV0::Settled { .. }
        ) {
            return Ok(());
        }
    }
}

fn graph() -> Result<(ReactiveEngineV0, Vec<ReactiveNodeIdV0>), Box<dyn Error>> {
    let policy = ChangePolicyV0::exact("performanceSemanticValue");
    let mut graph = ReactiveGraphBuilderV0::new();
    let mut inputs = Vec::with_capacity(INPUT_COUNT);
    let mut projections = Vec::with_capacity(INPUT_COUNT);
    for index in 0..INPUT_COUNT {
        let input = graph.add_input(
            ReactiveStateV0::available(ReactiveValueV0::Counter(index as u64)),
            policy,
        );
        inputs.push(input);
        projections.push((
            format!("projection-{index}"),
            graph.add_map(input, project_counter, policy),
        ));
    }
    let fold = graph.add_delta_fold(projections, policy)?;
    let boundary = graph.add_effect_boundary(fold, "performance-observation", policy);
    let mut engine = graph.build()?;
    engine.observe(boundary)?;
    settle(&mut engine)?;
    engine.drain_effect_receipts();
    Ok((engine, inputs))
}

fn measure(changes_input: bool) -> Result<Vec<Duration>, Box<dyn Error>> {
    let (mut engine, inputs) = graph()?;
    let mut samples = Vec::with_capacity(EVENT_COUNT);
    for event in 0..EVENT_COUNT {
        let index = event % INPUT_COUNT;
        let value = if changes_input {
            (index + event + 1) as u64
        } else {
            index as u64
        };
        let started = Instant::now();
        engine.deposit(
            inputs[index],
            ReactiveStateV0::available(ReactiveValueV0::Counter(value)),
        )?;
        settle(&mut engine)?;
        engine.drain_effect_receipts();
        samples.push(started.elapsed());
    }
    Ok(samples)
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = (sorted.len() - 1) * percentile / 100;
    sorted[index]
}

fn maximum(samples: &[Duration]) -> Duration {
    samples.iter().copied().max().unwrap_or_default()
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn main() -> Result<(), Box<dyn Error>> {
    let changed_input = measure(true)?;
    let unchanged_input = measure(false)?;
    let changed_p95 = percentile(&changed_input, 95);
    let changed_max = maximum(&changed_input);
    let unchanged_p95 = percentile(&unchanged_input, 95);
    let unchanged_max = maximum(&unchanged_input);

    println!(
        concat!(
            "{{\"schemaVersion\":\"omena.reactive-engine-step-envelope.v0\",",
            "\"inputCount\":{},\"eventCount\":{},",
            "\"p95CeilingMs\":{:.3},\"maxCeilingMs\":{:.3},",
            "\"changedInput\":{{\"p95Ms\":{:.3},\"maxMs\":{:.3}}},",
            "\"unchangedInput\":{{\"p95Ms\":{:.3},\"maxMs\":{:.3}}}}}"
        ),
        INPUT_COUNT,
        EVENT_COUNT,
        milliseconds(P95_CEILING),
        milliseconds(MAX_CEILING),
        milliseconds(changed_p95),
        milliseconds(changed_max),
        milliseconds(unchanged_p95),
        milliseconds(unchanged_max),
    );

    if changed_p95 > P95_CEILING
        || changed_max > MAX_CEILING
        || unchanged_p95 > P95_CEILING
        || unchanged_max > MAX_CEILING
    {
        return Err("reactive engine stepping exceeded its latency envelope".into());
    }
    Ok(())
}
